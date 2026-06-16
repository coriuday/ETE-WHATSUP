use axum::{routing::get, Json, Router};
use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::RequireOrgViewer,
    models::pagination::ApiResponse,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_messages))
        .with_state(state)
}

async fn list_messages(RequireOrgViewer(auth): RequireOrgViewer) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let _org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "messages": [] }))))
}
