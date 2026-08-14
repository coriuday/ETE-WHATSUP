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
pub mod automations;
pub mod dev_mock;

use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;

use crate::AppState;

/// Liveness — process is up (no dependency checks)
async fn health_live() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "WhatsUp API",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Readiness — DB + Redis must respond
async fn health_ready(State(state): State<AppState>) -> Result<Json<serde_json::Value>, crate::AppError> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|_| crate::AppError::ServiceUnavailable)?;

    let mut conn = state
        .redis
        .get()
        .await
        .map_err(|_| crate::AppError::ServiceUnavailable)?;
    let _: String = redis::cmd("PING")
        .query_async(&mut conn)
        .await
        .map_err(|_| crate::AppError::ServiceUnavailable)?;

    Ok(Json(json!({
        "status": "ready",
        "service": "WhatsUp API",
        "checks": { "database": "ok", "redis": "ok" }
    })))
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
        .nest("/automations", automations::router(state.clone()))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::rate_limit::api_rate_limit,
        ));

    let mut router = Router::new()
        .route("/api/v1/health", get(health_live))
        .route("/api/v1/health/live", get(health_live))
        .route(
            "/api/v1/health/ready",
            get(health_ready).with_state(state.clone()),
        )
        .nest(
            "/api/v1/auth",
            auth::router(state.clone()).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::rate_limit::auth_rate_limit,
            )),
        )
        .nest("/api/v1", api_routes)
        .nest(
            "/api/v1/webhooks",
            webhooks::router(state.clone()).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::rate_limit::webhook_rate_limit,
            )),
        );

    if state.config.enable_mock_provider {
        router = router.nest("/api/v1/dev/mock", dev_mock::router(state.clone()));
    }

    router
}
