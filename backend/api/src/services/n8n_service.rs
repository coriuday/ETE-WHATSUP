use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::AppState;

/// n8n integration service — fires webhook triggers for automation events
pub struct N8nService<'a> {
    state: &'a AppState,
    client: Client,
}

/// Standard events that trigger n8n workflows
#[derive(Debug, Clone)]
pub enum N8nEvent {
    ContactCreated { contact_id: Uuid, org_id: Uuid },
    ContactImported { import_id: Uuid, org_id: Uuid, count: i32 },
    CampaignLaunched { campaign_id: Uuid, org_id: Uuid },
    CampaignCompleted { campaign_id: Uuid, org_id: Uuid, sent: i32, delivered: i32 },
    MessageReceived { contact_id: Uuid, org_id: Uuid, message_id: Uuid },
    LeadOptedIn { contact_id: Uuid, org_id: Uuid },
}

impl N8nEvent {
    pub fn event_type(&self) -> &str {
        match self {
            N8nEvent::ContactCreated { .. } => "contact.created",
            N8nEvent::ContactImported { .. } => "contact.imported",
            N8nEvent::CampaignLaunched { .. } => "campaign.launched",
            N8nEvent::CampaignCompleted { .. } => "campaign.completed",
            N8nEvent::MessageReceived { .. } => "message.received",
            N8nEvent::LeadOptedIn { .. } => "lead.opted_in",
        }
    }

    pub fn payload(&self) -> Value {
        match self {
            N8nEvent::ContactCreated { contact_id, org_id } => json!({
                "contact_id": contact_id, "org_id": org_id
            }),
            N8nEvent::ContactImported { import_id, org_id, count } => json!({
                "import_id": import_id, "org_id": org_id, "count": count
            }),
            N8nEvent::CampaignLaunched { campaign_id, org_id } => json!({
                "campaign_id": campaign_id, "org_id": org_id
            }),
            N8nEvent::CampaignCompleted { campaign_id, org_id, sent, delivered } => json!({
                "campaign_id": campaign_id, "org_id": org_id,
                "sent": sent, "delivered": delivered
            }),
            N8nEvent::MessageReceived { contact_id, org_id, message_id } => json!({
                "contact_id": contact_id, "org_id": org_id, "message_id": message_id
            }),
            N8nEvent::LeadOptedIn { contact_id, org_id } => json!({
                "contact_id": contact_id, "org_id": org_id
            }),
        }
    }
}

impl<'a> N8nService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state, client: Client::new() }
    }

    /// Fire an event to all matching n8n automation triggers for the org
    pub async fn fire_event(&self, org_id: Uuid, event: N8nEvent) {
        let event_type = event.event_type();

        // Fetch all active triggers for this org and event type
        let triggers = sqlx::query!(
            "SELECT n8n_webhook_url FROM automation_triggers WHERE organization_id = $1 AND event_type = $2 AND is_active = TRUE",
            org_id, event_type
        )
        .fetch_all(&self.state.db)
        .await;

        let Ok(triggers) = triggers else {
            tracing::error!("Failed to fetch automation triggers");
            return;
        };

        let payload = json!({
            "event": event_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": event.payload()
        });

        for trigger in triggers {
            let client = self.client.clone();
            let url = trigger.n8n_webhook_url.clone();
            let body = payload.clone();

            // Fire-and-forget in background task
            tokio::spawn(async move {
                if let Err(e) = client.post(&url).json(&body).send().await {
                    tracing::error!("Failed to trigger n8n webhook {}: {}", url, e);
                } else {
                    tracing::debug!("Triggered n8n webhook: {}", url);
                }
            });

            // Update trigger stats
            let _ = sqlx::query!(
                "UPDATE automation_triggers SET last_triggered_at = NOW(), trigger_count = trigger_count + 1 WHERE n8n_webhook_url = $1",
                trigger.n8n_webhook_url
            )
            .execute(&self.state.db)
            .await;
        }
    }
}
