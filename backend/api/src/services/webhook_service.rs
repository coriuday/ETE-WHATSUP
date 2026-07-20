use serde_json::Value;
use uuid::Uuid;

use crate::{
    models::message::{MetaInboundMessage, MetaMessageStatus, MetaWebhookPayload},
    AppState,
};

/// Processes Meta WhatsApp webhooks with durable event logging and idempotency.
pub struct WebhookService<'a> {
    state: &'a AppState,
}

impl<'a> WebhookService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn process_payload(&self, payload: &MetaWebhookPayload) -> anyhow::Result<()> {
        for entry in &payload.entry {
            for change in &entry.changes {
                if change.field != "messages" {
                    continue;
                }

                let value = &change.value;
                let phone_number_id = value
                    .metadata
                    .as_ref()
                    .map(|m| m.phone_number_id.clone())
                    .unwrap_or_default();

                if let Some(messages) = &value.messages {
                    for msg in messages {
                        if let Err(e) = self.process_inbound(&phone_number_id, msg).await {
                            tracing::error!("Failed to process inbound message: {:?}", e);
                        }
                    }
                }

                if let Some(statuses) = &value.statuses {
                    for status in statuses {
                        if let Err(e) = self.process_status(status).await {
                            tracing::error!("Failed to process status update: {:?}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn claim_event(
        &self,
        external_id: &str,
        event_type: &str,
        account_id: Option<Uuid>,
        org_id: Option<Uuid>,
        payload: &Value,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO wa_webhook_events
                (account_id, organization_id, event_type, external_id, payload, processed)
            SELECT $1, $2, $3, $4, $5, FALSE
            WHERE NOT EXISTS (
                SELECT 1 FROM wa_webhook_events WHERE external_id = $4
            )
            RETURNING id
            "#,
        )
        .bind(account_id)
        .bind(org_id)
        .bind(event_type)
        .bind(external_id)
        .bind(payload)
        .fetch_optional(&self.state.db)
        .await?;

        Ok(result.is_some())
    }

    async fn mark_processed(&self, external_id: &str, error: Option<&str>) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE wa_webhook_events
            SET processed = TRUE, processed_at = NOW(), error = $2
            WHERE external_id = $1
            "#,
        )
        .bind(external_id)
        .bind(error)
        .execute(&self.state.db)
        .await?;
        Ok(())
    }

    async fn process_inbound(
        &self,
        phone_number_id: &str,
        msg: &MetaInboundMessage,
    ) -> anyhow::Result<()> {
        tracing::info!("Inbound message from {} (wamid: {})", msg.from, msg.id);

        let wa_account = sqlx::query_as::<_, (Uuid, Uuid)>(
            "SELECT id, organization_id FROM whatsapp_accounts WHERE phone_number_id = $1",
        )
        .bind(phone_number_id)
        .fetch_optional(&self.state.db)
        .await?;

        let Some((account_id, organization_id)) = wa_account else {
            tracing::warn!("No WA account found for phone_number_id: {}", phone_number_id);
            return Ok(());
        };

        let payload = serde_json::to_value(msg)?;
        let external_id = format!("msg:{}", msg.id);
        if !self
            .claim_event(
                &external_id,
                "message",
                Some(account_id),
                Some(organization_id),
                &payload,
            )
            .await?
        {
            tracing::debug!("Skipping duplicate inbound message {}", msg.id);
            return Ok(());
        }

        let result = self
            .store_inbound_message(&account_id, organization_id, msg)
            .await;

        match &result {
            Ok(()) => self.mark_processed(&external_id, None).await?,
            Err(e) => {
                self.mark_processed(&external_id, Some(&e.to_string()))
                    .await?;
            }
        }
        result
    }

    async fn store_inbound_message(
        &self,
        account_id: &Uuid,
        organization_id: Uuid,
        msg: &MetaInboundMessage,
    ) -> anyhow::Result<()> {
        let contact_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO contacts (organization_id, phone_number, source)
            VALUES ($1, $2, 'whatsapp_inbound')
            ON CONFLICT (organization_id, phone_number) DO UPDATE
                SET last_replied_at = NOW(), total_msgs_received = contacts.total_msgs_received + 1
            RETURNING id
            "#,
        )
        .bind(organization_id)
        .bind(format!("+{}", msg.from))
        .fetch_one(&self.state.db)
        .await?;

        let _ = sqlx::query(
            r#"
            INSERT INTO conversations
                (organization_id, wa_account_id, contact_id, status, is_in_session,
                 session_expires_at, first_message_at, last_message_at)
            VALUES ($1, $2, $3, 'open', TRUE, NOW() + INTERVAL '24 hours', NOW(), NOW())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(organization_id)
        .bind(*account_id)
        .bind(contact_id)
        .execute(&self.state.db)
        .await?;

        sqlx::query(
            r#"
            UPDATE conversations
            SET last_message_at = NOW(),
                last_message_body = $2,
                last_message_dir = 'inbound',
                unread_count = unread_count + 1,
                is_in_session = TRUE,
                session_expires_at = NOW() + INTERVAL '24 hours'
            WHERE contact_id = $1 AND organization_id = $3 AND status != 'resolved'
            "#,
        )
        .bind(contact_id)
        .bind(msg.text.as_ref().map(|t| t.body.clone()).unwrap_or_default())
        .bind(organization_id)
        .execute(&self.state.db)
        .await?;

        let body = msg.text.as_ref().map(|t| t.body.clone());
        sqlx::query(
            r#"
            INSERT INTO messages (
                organization_id, wa_account_id, contact_id,
                wa_message_id, direction, type, body, status
            )
            SELECT $1, $2, $3, $4, 'inbound', 'text', $5, 'delivered'
            WHERE NOT EXISTS (SELECT 1 FROM messages WHERE wa_message_id = $4)
            "#,
        )
        .bind(organization_id)
        .bind(*account_id)
        .bind(contact_id)
        .bind(&msg.id)
        .bind(body)
        .execute(&self.state.db)
        .await?;

        Ok(())
    }

    async fn process_status(&self, status: &MetaMessageStatus) -> anyhow::Result<()> {
        tracing::debug!("Status update for wamid {}: {}", status.id, status.status);

        let external_id = format!("status:{}:{}", status.id, status.status);
        let payload = serde_json::to_value(status)?;

        if !self
            .claim_event(&external_id, "status", None, None, &payload)
            .await?
        {
            tracing::debug!("Skipping duplicate status {} for {}", status.status, status.id);
            return Ok(());
        }

        let result = self.apply_status_update(status).await;
        match &result {
            Ok(()) => self.mark_processed(&external_id, None).await?,
            Err(e) => {
                self.mark_processed(&external_id, Some(&e.to_string()))
                    .await?;
            }
        }
        result
    }

    async fn apply_status_update(&self, status: &MetaMessageStatus) -> anyhow::Result<()> {
        match status.status.as_str() {
            "sent" | "delivered" | "read" | "failed" => {}
            _ => return Ok(()),
        };

        let updated = sqlx::query_as::<_, (Option<Uuid>,)>(
            r#"
            UPDATE messages
            SET status = $2::message_status,
                sent_at = CASE WHEN $2 = 'sent' AND sent_at IS NULL THEN NOW() ELSE sent_at END,
                delivered_at = CASE WHEN $2 = 'delivered' AND delivered_at IS NULL THEN NOW() ELSE delivered_at END,
                read_at = CASE WHEN $2 = 'read' AND read_at IS NULL THEN NOW() ELSE read_at END,
                failed_at = CASE WHEN $2 = 'failed' AND failed_at IS NULL THEN NOW() ELSE failed_at END
            WHERE wa_message_id = $1
              AND (
                CASE status::text
                    WHEN 'queued' THEN 0
                    WHEN 'sending' THEN 1
                    WHEN 'sent' THEN 2
                    WHEN 'delivered' THEN 3
                    WHEN 'read' THEN 4
                    WHEN 'failed' THEN 5
                    WHEN 'rejected' THEN 5
                    ELSE 0
                END
              ) < (
                CASE $2
                    WHEN 'sent' THEN 2
                    WHEN 'delivered' THEN 3
                    WHEN 'read' THEN 4
                    WHEN 'failed' THEN 5
                    ELSE 0
                END
              )
            RETURNING campaign_id
            "#,
        )
        .bind(&status.id)
        .bind(&status.status)
        .fetch_optional(&self.state.db)
        .await?;

        let Some((campaign_id,)) = updated else {
            return Ok(());
        };

        if let Some(campaign_id) = campaign_id {
            if status.status == "delivered" {
                sqlx::query(
                    "UPDATE campaigns SET delivered_count = delivered_count + 1 WHERE id = $1",
                )
                .bind(campaign_id)
                .execute(&self.state.db)
                .await?;
            } else if status.status == "read" {
                sqlx::query("UPDATE campaigns SET read_count = read_count + 1 WHERE id = $1")
                    .bind(campaign_id)
                    .execute(&self.state.db)
                    .await?;
            }
        }

        Ok(())
    }
}
