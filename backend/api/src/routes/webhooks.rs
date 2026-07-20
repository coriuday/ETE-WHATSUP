use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    models::message::MetaWebhookPayload,
    services::webhook_service::WebhookService,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/whatsapp", get(verify_webhook).post(receive_webhook))
        .with_state(state)
}

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

async fn receive_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body_bytes: axum::body::Bytes,
) -> AppResult<StatusCode> {
    if let Some(signature_header) = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
    {
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
    } else if !state.config.meta_wa_app_secret.is_empty() {
        tracing::warn!("Missing x-hub-signature-256 header");
        return Err(AppError::Forbidden);
    }

    let payload: MetaWebhookPayload = serde_json::from_slice(&body_bytes)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    tracing::debug!("Received WhatsApp webhook: {} entries", payload.entry.len());

    // Process after ACK path: still sync for correctness, but idempotent + persisted
    let service = WebhookService::new(&state);
    if let Err(e) = service.process_payload(&payload).await {
        tracing::error!("Webhook processing error: {:?}", e);
    }

    Ok(StatusCode::OK)
}
