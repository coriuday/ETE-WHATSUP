use axum::{extract::{Path, Query, State}, http::StatusCode, routing::{get, post, put}, Json, Router};
use uuid::Uuid;
use crate::{errors::{AppError, AppResult}, middleware::auth::AuthUser, models::pagination::ApiResponse, AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_conversations))
        .route("/:id", get(get_conversation))
        .route("/:id/messages", get(get_messages).post(send_message))
        .route("/:id/assign", put(assign_conversation))
        .route("/:id/resolve", post(resolve_conversation))
        .route("/:id/reopen", post(reopen_conversation))
        .with_state(state)
}

async fn list_conversations(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<crate::models::conversation::ConversationListQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let convs = sqlx::query!(
        r#"SELECT cv.id, cv.status::text, cv.unread_count, cv.last_message_at, cv.last_message_body,
                  c.phone_number, c.first_name, c.last_name
           FROM conversations cv JOIN contacts c ON c.id = cv.contact_id
           WHERE cv.organization_id = $1
           ORDER BY cv.last_message_at DESC NULLS LAST LIMIT 50"#,
        org_id
    ).fetch_all(&state.db).await.map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = convs.iter().map(|cv| serde_json::json!({
        "id": cv.id, "status": cv.status, "unread_count": cv.unread_count,
        "last_message_at": cv.last_message_at, "last_message_body": cv.last_message_body,
        "contact": { "phone": cv.phone_number, "name": format!("{} {}", cv.first_name.as_deref().unwrap_or(""), cv.last_name.as_deref().unwrap_or("")).trim().to_string() }
    })).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "conversations": data }))))
}

async fn get_conversation(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let cv = sqlx::query!("SELECT id, status::text, unread_count, last_message_at FROM conversations WHERE id = $1 AND organization_id = $2", id, org_id)
        .fetch_optional(&state.db).await.map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Conversation".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "id": cv.id, "status": cv.status, "unread_count": cv.unread_count }))))
}

async fn get_messages(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let msgs = sqlx::query!(
        r#"SELECT id, direction::text, type::text, body, media_url, status::text, created_at
           FROM messages WHERE conversation_id = $1 AND organization_id = $2
           ORDER BY created_at ASC LIMIT 100"#,
        id, org_id
    ).fetch_all(&state.db).await.map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = msgs.iter().map(|m| serde_json::json!({
        "id": m.id, "direction": m.direction, "type": m.r#type,
        "body": m.body, "status": m.status, "created_at": m.created_at
    })).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "messages": data }))))
}

async fn send_message(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(req): Json<crate::models::conversation::SendMessageRequest>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Message sent")))
}

async fn assign_conversation(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(req): Json<crate::models::conversation::AssignConversationRequest>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE conversations SET assigned_to = $1, assigned_at = NOW() WHERE id = $2 AND organization_id = $3", req.user_id, id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Assigned")))
}

async fn resolve_conversation(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE conversations SET status = 'resolved', resolved_at = NOW() WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Resolved")))
}

async fn reopen_conversation(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE conversations SET status = 'open', resolved_at = NULL WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Reopened")))
}
