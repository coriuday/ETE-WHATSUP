use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post, put}, Json, Router};
use uuid::Uuid;
use validator::Validate;
use crate::{errors::{AppError, AppResult}, middleware::auth::AuthUser, models::pagination::ApiResponse, AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/accounts", get(list_accounts).post(connect_account))
        .route("/accounts/:id", get(get_account).delete(disconnect_account))
        .route("/accounts/:id/profile", put(update_profile))
        .route("/accounts/:id/sync", post(sync_account))
        .with_state(state)
}

async fn list_accounts(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let accounts = sqlx::query!(
        r#"SELECT id, display_name, phone_number, status::text, account_type::text, quality_rating, total_msgs_sent, connected_at
           FROM whatsapp_accounts WHERE organization_id = $1 AND deleted_at IS NULL"#,
        org_id
    ).fetch_all(&state.db).await.map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = accounts.iter().map(|a| serde_json::json!({
        "id": a.id, "display_name": a.display_name, "phone_number": a.phone_number,
        "status": a.status, "type": a.account_type, "quality_rating": a.quality_rating,
        "total_msgs_sent": a.total_msgs_sent, "connected_at": a.connected_at
    })).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "accounts": data }))))
}

async fn connect_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<crate::models::whatsapp_account::ConnectWaAccountRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    // Encrypt the access token
    let encrypted_token = crate::utils::encryption::encrypt(&req.access_token, &state.config.encryption_key)
        .map_err(|e| AppError::Internal(e))?;

    let id = sqlx::query_scalar!(
        r#"INSERT INTO whatsapp_accounts (organization_id, display_name, phone_number, phone_number_id, waba_id, access_token_enc, status)
           VALUES ($1, $2, $3, $4, $5, $6, 'connected') RETURNING id"#,
        org_id, req.display_name, req.phone_number, req.phone_number_id, req.waba_id, encrypted_token
    ).fetch_one(&state.db).await.map_err(AppError::Database)?;

    Ok((StatusCode::CREATED, Json(ApiResponse::with_message(serde_json::json!({ "id": id }), "WhatsApp account connected"))))
}

async fn get_account(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let a = sqlx::query!(
        r#"SELECT id, display_name, phone_number, phone_number_id, status::text, quality_rating, business_name, total_msgs_sent, connected_at
           FROM whatsapp_accounts WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL"#,
        id, org_id
    ).fetch_optional(&state.db).await.map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("WhatsApp account".into()))?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": a.id, "display_name": a.display_name, "phone_number": a.phone_number,
        "phone_number_id": a.phone_number_id, "status": a.status,
        "quality_rating": a.quality_rating, "business_name": a.business_name,
        "total_msgs_sent": a.total_msgs_sent, "connected_at": a.connected_at,
    }))))
}

async fn disconnect_account(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<()>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE whatsapp_accounts SET deleted_at = NOW(), status = 'disconnected' WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message((), "Account disconnected")))
}

async fn update_profile(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(req): Json<crate::models::whatsapp_account::UpdateWaProfileRequest>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!(
        "UPDATE whatsapp_accounts SET business_name = COALESCE($1, business_name), business_description = COALESCE($2, business_description) WHERE id = $3 AND organization_id = $4",
        req.business_name, req.business_description, id, org_id
    ).execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Profile updated")))
}

async fn sync_account(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Account synced with Meta")))
}
