use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::{RequireOrgAdmin, RequireOrgViewer},
    models::pagination::ApiResponse,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_workflows).post(create_workflow))
        .route("/:id", get(get_workflow).put(update_workflow).delete(delete_workflow))
        .route("/runs", get(list_runs))
        .with_state(state)
}

#[derive(Deserialize)]
struct WorkflowBody {
    name: String,
    description: Option<String>,
    trigger_type: String,
    definition: Option<serde_json::Value>,
    enabled: Option<bool>,
}

async fn list_workflows(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let rows = sqlx::query(
        r#"SELECT id, name, description, enabled, trigger_type, definition, created_at
           FROM automation_workflows WHERE organization_id = $1 AND deleted_at IS NULL
           ORDER BY created_at DESC"#,
    )
    .bind(org_id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "workflows": rows.iter().map(|r| serde_json::json!({
            "id": r.get::<Uuid, _>("id"),
            "name": r.get::<String, _>("name"),
            "description": r.try_get::<Option<String>, _>("description").ok().flatten(),
            "enabled": r.get::<bool, _>("enabled"),
            "trigger_type": r.get::<String, _>("trigger_type"),
            "definition": r.try_get::<serde_json::Value, _>("definition").ok(),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
        })).collect::<Vec<_>>()
    }))))
}

async fn create_workflow(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(req): Json<WorkflowBody>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let definition = req.definition.unwrap_or_else(|| serde_json::json!({ "steps": [] }));
    let id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO automation_workflows (organization_id, name, description, trigger_type, definition, enabled)
           VALUES ($1, $2, $3, $4, $5, TRUE) RETURNING id"#,
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.trigger_type)
    .bind(&definition)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({ "id": id }))),
    ))
}

async fn get_workflow(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let r = sqlx::query(
        r#"SELECT id, name, description, enabled, trigger_type, definition, created_at
           FROM automation_workflows WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(org_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Workflow".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "description": r.try_get::<Option<String>, _>("description").ok().flatten(),
        "enabled": r.get::<bool, _>("enabled"),
        "trigger_type": r.get::<String, _>("trigger_type"),
        "definition": r.try_get::<serde_json::Value, _>("definition").ok(),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
    }))))
}

async fn update_workflow(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<WorkflowBody>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let definition = req.definition.unwrap_or_else(|| serde_json::json!({ "steps": [] }));
    sqlx::query(
        r#"UPDATE automation_workflows
           SET name = $3, description = $4, trigger_type = $5, definition = $6, enabled = COALESCE($7, enabled)
           WHERE id = $1 AND organization_id = $2"#,
    )
    .bind(id)
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.trigger_type)
    .bind(&definition)
    .bind(req.enabled)
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({ "id": id }), "Updated")))
}

async fn delete_workflow(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query(
        "UPDATE automation_workflows SET deleted_at = NOW() WHERE id = $1 AND organization_id = $2",
    )
    .bind(id)
    .bind(org_id)
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message((), "Deleted")))
}

#[derive(Deserialize)]
struct RunsQuery {
    workflow_id: Option<Uuid>,
}

async fn list_runs(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(q): Query<RunsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let rows = sqlx::query(
        r#"SELECT id, workflow_id, status, trigger_type, error, started_at, finished_at
           FROM automation_runs
           WHERE organization_id = $1 AND ($2::uuid IS NULL OR workflow_id = $2)
           ORDER BY started_at DESC LIMIT 100"#,
    )
    .bind(org_id)
    .bind(q.workflow_id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "runs": rows.iter().map(|r| serde_json::json!({
            "id": r.get::<Uuid, _>("id"),
            "workflow_id": r.get::<Uuid, _>("workflow_id"),
            "status": r.get::<String, _>("status"),
            "trigger_type": r.get::<String, _>("trigger_type"),
            "error": r.try_get::<Option<String>, _>("error").ok().flatten(),
            "started_at": r.get::<chrono::DateTime<chrono::Utc>, _>("started_at"),
            "finished_at": r.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("finished_at").ok().flatten()
        })).collect::<Vec<_>>()
    }))))
}
