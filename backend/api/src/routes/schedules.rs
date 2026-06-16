use axum::{extract::{Path, State}, http::StatusCode, routing::{delete, get, post}, Json, Router};
use uuid::Uuid;
use chrono::Utc;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::{RequireOrgAdmin, RequireOrgViewer},
    models::pagination::ApiResponse,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_schedules).post(create_schedule))
        .route("/:id", get(get_schedule).delete(delete_schedule))
        .route("/:id/pause", post(pause_schedule))
        .route("/:id/resume", post(resume_schedule))
        .with_state(state)
}

async fn list_schedules(State(state): State<AppState>, RequireOrgViewer(auth): RequireOrgViewer) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
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

async fn create_schedule(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(body): Json<serde_json::Value>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    let campaign_id = body["campaign_id"]
        .as_str()
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::Validation("campaign_id is required".into()))?;

    let frequency = body["frequency"]
        .as_str()
        .unwrap_or("once")
        .to_string();

    let next_run_at_str = body["next_run_at"]
        .as_str()
        .ok_or_else(|| AppError::Validation("next_run_at is required".into()))?;

    let next_run_at = chrono::DateTime::parse_from_rfc3339(next_run_at_str)
        .map_err(|_| AppError::Validation("next_run_at must be a valid ISO 8601 datetime".into()))?
        .with_timezone(&chrono::Utc);

    if next_run_at <= chrono::Utc::now() {
        return Err(AppError::Validation("next_run_at must be in the future".into()));
    }

    // Verify campaign belongs to org
    let campaign_exists = sqlx::query_scalar!(
        "SELECT id FROM campaigns WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
        campaign_id, org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;

    if campaign_exists.is_none() {
        return Err(AppError::NotFound("Campaign".into()));
    }

    let cron_expression = body["cron_expression"].as_str().map(|s| s.to_string());
    let timezone = body["timezone"].as_str().unwrap_or("Asia/Kolkata").to_string();
    let max_runs = body["max_runs"].as_i64().map(|n| n as i32);
    let ends_at: Option<chrono::DateTime<chrono::Utc>> = body["ends_at"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO campaign_schedules
            (organization_id, campaign_id, frequency, cron_expression, timezone,
             next_run_at, max_runs, ends_at, status, created_by)
        VALUES ($1, $2, $3::schedule_frequency, $4, $5, $6, $7, $8, 'active', $9)
        RETURNING id
        "#,
        org_id,
        campaign_id,
        frequency as _,
        cron_expression,
        timezone,
        next_run_at,
        max_runs,
        ends_at,
        auth.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({
            "id": id,
            "campaign_id": campaign_id,
            "frequency": frequency,
            "next_run_at": next_run_at,
            "status": "active"
        }))),
    ))
}

async fn get_schedule(State(state): State<AppState>, RequireOrgViewer(auth): RequireOrgViewer, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let _org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "id": id }))))
}

async fn delete_schedule(State(state): State<AppState>, RequireOrgAdmin(auth): RequireOrgAdmin, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE campaign_schedules SET status = 'cancelled' WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message((), "Schedule cancelled")))
}

async fn pause_schedule(State(state): State<AppState>, RequireOrgAdmin(auth): RequireOrgAdmin, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE campaign_schedules SET status = 'paused' WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Schedule paused")))
}

async fn resume_schedule(State(state): State<AppState>, RequireOrgAdmin(auth): RequireOrgAdmin, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE campaign_schedules SET status = 'active' WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Schedule resumed")))
}
