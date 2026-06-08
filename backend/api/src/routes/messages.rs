use axum::{routing::get, Json, Router};
use crate::{errors::AppResult, models::pagination::ApiResponse, AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_messages))
        .with_state(state)
}

async fn list_messages() -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    Ok(Json(ApiResponse::ok(serde_json::json!({ "messages": [] }))))
}
