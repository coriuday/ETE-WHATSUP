use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::{RequireOrgAdmin, RequireOrgViewer},
    models::pagination::ApiResponse,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_templates).post(create_template))
        .route("/:id", get(get_template).put(update_template).delete(delete_template))
        .route("/:id/submit", post(submit_for_approval))
        .with_state(state)
}

async fn list_templates(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<crate::models::template::TemplateListQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let templates = sqlx::query!(
        r#"SELECT id, name, display_name, category::text, language::text, status::text, variable_count, usage_count, created_at
           FROM templates WHERE organization_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"#,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = templates.iter().map(|t| serde_json::json!({
        "id": t.id, "name": t.name, "display_name": t.display_name,
        "category": t.category, "language": t.language, "status": t.status,
        "variable_count": t.variable_count, "usage_count": t.usage_count,
        "created_at": t.created_at
    })).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "templates": data }))))
}

async fn create_template(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Json(req): Json<crate::models::template::CreateTemplateRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    use validator::Validate;
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    // Count variables in body_text
    let var_count = req.body_text.matches("{{").count() as i32;

    let id = sqlx::query_scalar!(
        r#"INSERT INTO templates (organization_id, wa_account_id, name, display_name, category, language, body_text, header, footer_text, buttons, variable_count, created_by)
           VALUES ($1, $2, $3, $4, CAST($5::text AS template_category), CAST($6::text AS template_language), $7, $8, $9, $10, $11, $12)
           RETURNING id"#,
        org_id, req.wa_account_id, req.name, req.display_name, req.category as _,
        req.language, req.body_text, req.header, req.footer_text, req.buttons, var_count, auth.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    crate::services::audit_service::audit_log(
        &state,
        "template.created",
        Some(auth.id),
        Some(org_id),
        Some("template"),
        Some(id),
        serde_json::json!({ "name": req.name, "language": req.language }),
    );

    Ok((StatusCode::CREATED, Json(ApiResponse::ok(serde_json::json!({ "id": id })))))
}

async fn get_template(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let t = sqlx::query!(
        r#"SELECT id, name, display_name, category::text, language::text, status::text, header, body_text, footer_text, buttons, variable_count, variable_definitions, created_at
           FROM templates WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL"#,
        id, org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Template".into()))?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": t.id, "name": t.name, "display_name": t.display_name,
        "category": t.category, "language": t.language, "status": t.status,
        "header": t.header, "body_text": t.body_text, "footer_text": t.footer_text,
        "buttons": t.buttons, "variable_count": t.variable_count,
        "variable_definitions": t.variable_definitions, "created_at": t.created_at,
    }))))
}

async fn update_template(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    crate::services::audit_service::audit_log(
        &state,
        "template.updated",
        Some(auth.id),
        Some(org_id),
        Some("template"),
        Some(id),
        serde_json::json!({}),
    );

    Ok(Json(ApiResponse::with_message(serde_json::json!({"id": id}), "Template updated")))
}

async fn delete_template(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE templates SET deleted_at = NOW() WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    crate::services::audit_service::audit_log(
        &state,
        "template.deleted",
        Some(auth.id),
        Some(org_id),
        Some("template"),
        Some(id),
        serde_json::json!({}),
    );

    Ok(Json(ApiResponse::with_message((), "Template deleted")))
}

async fn submit_for_approval(
    State(state): State<AppState>,
    RequireOrgAdmin(auth): RequireOrgAdmin,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!(
        "UPDATE templates SET status = 'pending_approval', submitted_at = NOW() WHERE id = $1 AND organization_id = $2",
        id, org_id
    )
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    crate::services::audit_service::audit_log(
        &state,
        "template.submitted",
        Some(auth.id),
        Some(org_id),
        Some("template"),
        Some(id),
        serde_json::json!({}),
    );

    Ok(Json(ApiResponse::with_message(serde_json::json!({"id": id}), "Template submitted for Meta approval")))
}
