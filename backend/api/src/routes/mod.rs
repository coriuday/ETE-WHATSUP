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
    let api_routes = Router::new()
        .nest("/organizations", organizations::router(state.clone()))
        .nest("/contacts", contacts::router(state.clone()))
        .nest("/campaigns", campaigns::router(state.clone()))
        .nest("/templates", templates::router(state.clone()))
        .nest("/conversations", conversations::router(state.clone()))
        .nest("/whatsapp", whatsapp::router(state.clone()))
        .nest("/analytics", analytics::router(state.clone()))
        .nest("/schedules", schedules::router(state.clone()))
        .nest("/messages", messages::router(state.clone()))
        .layer(axum::middleware::from_fn_with_state(state.clone(), crate::middleware::rate_limit::api_rate_limit));

    Router::new()
        // Health check (no auth, no rate limiting)
        .route("/api/v1/health", get(health))
        // Auth routes (auth rate limit)
        .nest(
            "/api/v1/auth",
            auth::router(state.clone())
                .layer(axum::middleware::from_fn_with_state(state.clone(), crate::middleware::rate_limit::auth_rate_limit))
        )
        // API routes (API rate limit)
        .nest("/api/v1", api_routes)
        // Webhooks (webhook rate limit)
        .nest(
            "/api/v1/webhooks",
            webhooks::router(state.clone())
                .layer(axum::middleware::from_fn_with_state(state.clone(), crate::middleware::rate_limit::webhook_rate_limit))
        )
}
