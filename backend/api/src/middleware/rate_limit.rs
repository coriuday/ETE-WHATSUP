use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use redis::AsyncCommands;
use serde_json::json;

use crate::cache::RedisPool;

/// Simple Redis-backed sliding window rate limiter
/// Key: rate_limit:{ip}  Value: request count  TTL: 60 seconds
pub async fn rate_limit_middleware(
    State(redis_pool): State<Arc<RedisPool>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .trim()
        .to_string();

    let key = format!("rate_limit:{}", ip);

    let result: Result<Option<u64>, _> = async {
        let mut conn = redis_pool
            .get()
            .await
            .map_err(|e| e.to_string())?;

        let count: u64 = conn.incr(&key, 1u64).await.map_err(|e| e.to_string())?;

        if count == 1 {
            // First request in window — set 60 second expiry
            let _: () = conn.expire(&key, 60).await.map_err(|e| e.to_string())?;
        }

        Ok(Some(count))
    }
    .await;

    match result {
        Ok(Some(count)) if count > 100 => {
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
