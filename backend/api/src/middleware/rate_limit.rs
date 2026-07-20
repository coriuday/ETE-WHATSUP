use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::{errors::AppError, AppState};

pub async fn auth_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config.rate_limit_auth_per_min;
    rate_limit_helper(state, request, next, "rate_limit:auth:", limit, 60).await
}

pub async fn api_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config.rate_limit_requests_per_min;
    rate_limit_helper(state, request, next, "rate_limit:api:", limit, 60).await
}

pub async fn webhook_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config.rate_limit_webhook_per_min;
    rate_limit_helper(state, request, next, "rate_limit:webhook:", limit, 60).await
}

async fn rate_limit_helper(
    state: AppState,
    request: Request<Body>,
    next: Next,
    prefix: &str,
    limit: u64,
    window_secs: i64,
) -> Result<Response, AppError> {
    use deadpool_redis::redis::AsyncCommands;

    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .trim()
        .to_string();

    let key = format!("{}{}", prefix, ip);

    let count_result: Result<u64, String> = async {
        let mut conn = state.redis.get().await.map_err(|e| e.to_string())?;
        let count: u64 = conn.incr(&key, 1u64).await.map_err(|e| e.to_string())?;
        if count == 1 {
            let _: () = conn
                .expire(&key, window_secs)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(count)
    }
    .await;

    match count_result {
        Ok(count) if count > limit => Err(AppError::RateLimited),
        // Fail-open when Redis is unavailable (documented Phase 1 behavior)
        _ => Ok(next.run(request).await),
    }
}
