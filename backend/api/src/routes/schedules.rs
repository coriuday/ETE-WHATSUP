use axum::{extract::{Path, Query, State}, http::StatusCode, routing::{delete, get, post}, Json, Router};
use uuid::Uuid;
use crate::{errors::{AppError, AppResult}, middleware::auth::AuthUser, models::pagination::ApiResponse, AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_schedules).post(create_schedule))
        .route("/:id", get(get_schedule).delete(delete_schedule))
        .route("/:id/pause", post(pause_schedule))
        .route("/:id/resume", post(resume_schedule))
        .with_state(state)
}

async fn list_schedules(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let schedules = sqlx::query!(
        r#"SELECT cs.id, cs.frequency::text, cs.next_run_at, cs.last_run_at, cs.run_count, cs.status::text,
                  c.name as campaign_name
           FROM campaign_schedules cs JOIN campaigns c ON c.id = cs.campaign_id
           WHERE cs.organization_id = $1
           ORDER BY cs.next_run_at ASC"#,
        org_id
    ).fetch_all(&state.db).await.map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = schedules.iter().map(|s| serde_json::json!({
        "id": s.id, "frequency": s.frequency, "next_run_at": s.next_run_at,
        "last_run_at": s.last_run_at, "run_count": s.run_count, "status": s.status,
        "campaign_name": s.campaign_name,
    })).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "schedules": data }))))
}

async fn create_schedule(State(state): State<AppState>, auth: AuthUser, Json(body): Json<serde_json::Value>) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    Ok((StatusCode::CREATED, Json(ApiResponse::with_message(serde_json::json!({}), "Schedule created"))))
}

async fn get_schedule(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    Ok(Json(ApiResponse::ok(serde_json::json!({ "id": id }))))
}

async fn delete_schedule(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE campaign_schedules SET status = 'cancelled' WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message((), "Schedule cancelled")))
}

async fn pause_schedule(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE campaign_schedules SET status = 'paused' WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Schedule paused")))
}

async fn resume_schedule(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE campaign_schedules SET status = 'active' WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Schedule resumed")))
}
