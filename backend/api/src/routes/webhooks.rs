use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    models::{message::MetaWebhookPayload, pagination::ApiResponse},
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/whatsapp", get(verify_webhook).post(receive_webhook))
        .with_state(state)
}

/// GET /api/v1/webhooks/whatsapp — Meta webhook verification challenge
#[derive(Deserialize)]
struct WebhookVerifyQuery {
    #[serde(rename = "hub.mode")]
    mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    challenge: Option<String>,
}

async fn verify_webhook(
    State(state): State<AppState>,
    Query(query): Query<WebhookVerifyQuery>,
) -> AppResult<axum::response::Response<String>> {
    if query.mode.as_deref() == Some("subscribe")
        && query.verify_token.as_deref() == Some(&state.config.meta_wa_verify_token)
    {
        let challenge = query.challenge.clone().unwrap_or_default();
        tracing::info!("WhatsApp webhook verified successfully");
        Ok(axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(challenge)
            .unwrap())
    } else {
        tracing::warn!("WhatsApp webhook verification failed — invalid verify token");
        Err(AppError::Forbidden)
    }
}

/// POST /api/v1/webhooks/whatsapp — Receive webhook events from Meta
async fn receive_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body_bytes: axum::body::Bytes,
) -> AppResult<StatusCode> {
    // 1. Signature Verification
    if let Some(signature_header) = headers.get("x-hub-signature-256").and_then(|v| v.to_str().ok()) {
        if !signature_header.starts_with("sha256=") {
            return Err(AppError::Forbidden);
        }
        let expected_hex = &signature_header["sha256=".len()..];
        let key = state.config.meta_wa_app_secret.as_bytes();
        let computed_hex = crate::utils::encryption::hmac_sha256_hex(key, &body_bytes);

        if expected_hex != computed_hex {
            tracing::warn!("Meta webhook signature verification failed");
            return Err(AppError::Forbidden);
        }
    } else {
        // If app secret is set and header is missing, reject
        if !state.config.meta_wa_app_secret.is_empty() {
            tracing::warn!("Missing x-hub-signature-256 header");
            return Err(AppError::Forbidden);
        }
    }

    // 2. Parse payload JSON
    let payload: MetaWebhookPayload = serde_json::from_slice(&body_bytes)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    tracing::debug!("Received WhatsApp webhook: {} entries", payload.entry.len());

    for entry in &payload.entry {
        for change in &entry.changes {
            if change.field != "messages" {
                continue;
            }

            let value = &change.value;

            // Process inbound messages
            if let Some(messages) = &value.messages {
                for msg in messages {
                    let phone_number_id = value
                        .metadata
                        .as_ref()
                        .map(|m| m.phone_number_id.clone())
                        .unwrap_or_default();

                    if let Err(e) = process_inbound_message(&state, &phone_number_id, msg).await {
                        tracing::error!("Failed to process inbound message: {:?}", e);
                    }
                }
            }

            // Process delivery status updates
            if let Some(statuses) = &value.statuses {
                for status in statuses {
                    if let Err(e) = process_status_update(&state, status).await {
                        tracing::error!("Failed to process status update: {:?}", e);
                    }
                }
            }
        }
    }

    // Always return 200 to Meta — they retry on non-200
    Ok(StatusCode::OK)
}

async fn process_inbound_message(
    state: &AppState,
    phone_number_id: &str,
    msg: &crate::models::message::MetaInboundMessage,
) -> anyhow::Result<()> {
    tracing::info!("Inbound message from {} (wamid: {})", msg.from, msg.id);

    // Find the WA account by phone_number_id
    let wa_account = sqlx::query!(
        "SELECT id, organization_id FROM whatsapp_accounts WHERE phone_number_id = $1",
        phone_number_id
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(account) = wa_account else {
        tracing::warn!("No WA account found for phone_number_id: {}", phone_number_id);
        return Ok(());
    };

    // Find or create contact
    let contact = sqlx::query!(
        r#"
        INSERT INTO contacts (organization_id, phone_number, source)
        VALUES ($1, $2, 'whatsapp_inbound')
        ON CONFLICT (organization_id, phone_number) DO UPDATE
            SET last_replied_at = NOW(), total_msgs_received = contacts.total_msgs_received + 1
        RETURNING id
        "#,
        account.organization_id,
        format!("+{}", msg.from)
    )
    .fetch_one(&state.db)
    .await?;

    // Find or create conversation
    let conversation = sqlx::query!(
        r#"
        INSERT INTO conversations (organization_id, wa_account_id, contact_id, status, is_in_session, session_expires_at, first_message_at, last_message_at)
        VALUES ($1, $2, $3, 'open', TRUE, NOW() + INTERVAL '24 hours', NOW(), NOW())
        ON CONFLICT DO NOTHING
        RETURNING id
        "#,
        account.organization_id,
        account.id,
        contact.id
    )
    .fetch_optional(&state.db)
    .await?;

    // Upsert conversation update
    sqlx::query!(
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
        contact.id,
        msg.text.as_ref().map(|t| t.body.clone()).unwrap_or_default(),
        account.organization_id
    )
    .execute(&state.db)
    .await?;

    // Store the message
    let body = msg.text.as_ref().map(|t| t.body.clone());
    sqlx::query!(
        r#"
        INSERT INTO messages (
            organization_id, wa_account_id, contact_id,
            wa_message_id, direction, type, body, status
        ) VALUES ($1, $2, $3, $4, 'inbound', 'text', $5, 'delivered')
        "#,
        account.organization_id,
        account.id,
        contact.id,
        msg.id,
        body
    )
    .execute(&state.db)
    .await?;

    Ok(())
}

async fn process_status_update(
    state: &AppState,
    status: &crate::models::message::MetaMessageStatus,
) -> anyhow::Result<()> {
    tracing::debug!("Status update for wamid {}: {}", status.id, status.status);

    let (status_col_update, timestamp_update) = match status.status.as_str() {
        "sent" => ("sent", "sent_at = NOW()"),
        "delivered" => ("delivered", "delivered_at = NOW()"),
        "read" => ("read", "read_at = NOW()"),
        "failed" => ("failed", "failed_at = NOW()"),
        _ => return Ok(()),
    };

    // Update message status
    sqlx::query!(
        r#"
        UPDATE messages
        SET status = $2::message_status,
            sent_at = CASE WHEN $2 = 'sent' THEN NOW() ELSE sent_at END,
            delivered_at = CASE WHEN $2 = 'delivered' THEN NOW() ELSE delivered_at END,
            read_at = CASE WHEN $2 = 'read' THEN NOW() ELSE read_at END,
            failed_at = CASE WHEN $2 = 'failed' THEN NOW() ELSE failed_at END
        WHERE wa_message_id = $1
        "#,
        status.id,
        status.status as _
    )
    .execute(&state.db)
    .await?;

    // Update campaign counters
    if status.status == "delivered" {
        sqlx::query!(
            r#"
            UPDATE campaigns
            SET delivered_count = delivered_count + 1
            WHERE id = (
                SELECT campaign_id FROM messages WHERE wa_message_id = $1 LIMIT 1
            )
            "#,
            status.id
        )
        .execute(&state.db)
        .await?;
    } else if status.status == "read" {
        sqlx::query!(
            r#"
            UPDATE campaigns
            SET read_count = read_count + 1
            WHERE id = (
                SELECT campaign_id FROM messages WHERE wa_message_id = $1 LIMIT 1
            )
            "#,
            status.id
        )
        .execute(&state.db)
        .await?;
    }

    Ok(())
}
