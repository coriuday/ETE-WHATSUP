pub mod auth;
pub mod organizations;
pub mod contacts;
pub mod campaigns;
pub mod messages;
pub mod templates;
pub mod conversations;
pub mod whatsapp;
pub mod analytics;
pub mod schedules;
pub mod webhooks;

use axum::{routing::get, Json, Router};
use serde_json::json;

use crate::AppState;

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "WhatsUp API",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check (no auth)
        .route("/api/v1/health", get(health))
        // Auth routes
        .nest("/api/v1/auth", auth::router(state.clone()))
        // Organization routes
        .nest("/api/v1/organizations", organizations::router(state.clone()))
        // Contact routes
        .nest("/api/v1/contacts", contacts::router(state.clone()))
        // Campaign routes
        .nest("/api/v1/campaigns", campaigns::router(state.clone()))
        // Template routes
        .nest("/api/v1/templates", templates::router(state.clone()))
        // Conversation / Inbox routes
        .nest("/api/v1/conversations", conversations::router(state.clone()))
        // WhatsApp account management
        .nest("/api/v1/whatsapp", whatsapp::router(state.clone()))
        // Analytics
        .nest("/api/v1/analytics", analytics::router(state.clone()))
        // Schedules
        .nest("/api/v1/schedules", schedules::router(state.clone()))
        // Meta Webhooks (no JWT auth — uses verify token)
        .nest("/api/v1/webhooks", webhooks::router(state))
}
