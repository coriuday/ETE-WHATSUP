use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use deadpool_redis::redis::AsyncCommands;
use serde_json::json;

use crate::AppState;

pub async fn auth_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    rate_limit_helper(state, request, next, "rate_limit:auth:", 5, 60).await
}

pub async fn api_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    rate_limit_helper(state, request, next, "rate_limit:api:", 300, 60).await
}

pub async fn webhook_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    rate_limit_helper(state, request, next, "rate_limit:webhook:", 1000, 60).await
}

async fn rate_limit_helper(
    state: AppState,
    request: Request<Body>,
    next: Next,
    prefix: &str,
    limit: u64,
    window_secs: i64,
) -> Response {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .trim()
        .to_string();

    let key = format!("{}{}", prefix, ip);

    let result: Result<Option<u64>, String> = async {
        let mut conn = state.redis
            .get()
            .await
            .map_err(|e| e.to_string())?;

        let count: u64 = conn.incr(&key, 1u64).await.map_err(|e| e.to_string())?;

        if count == 1 {
            let _: () = conn.expire(&key, window_secs).await.map_err(|e| e.to_string())?;
        }

        Ok(Some(count))
    }
    .await;

    match result {
        Ok(Some(count)) if count > limit => {
            let body = Json(json!({
                "success": false,
                "error": {
                    "code": "RATE_LIMITED",
                    "message": "Too many requests. Please slow down."
                }
            }));
            (StatusCode::TOO_MANY_REQUESTS, body).into_response()
        }
        _ => next.run(request).await,
    }
}
