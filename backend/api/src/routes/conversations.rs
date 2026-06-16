use axum::{extract::{Path, Query, State}, http::StatusCode, routing::{get, post, put}, Json, Router};
use uuid::Uuid;
use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::{RequireOrgMember, RequireOrgViewer},
    models::{conversation::SendMessageRequest, pagination::ApiResponse},
    services::whatsapp_service::WhatsAppService,
    AppState,
};

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
    RequireOrgViewer(auth): RequireOrgViewer,
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

async fn get_conversation(State(state): State<AppState>, RequireOrgViewer(auth): RequireOrgViewer, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let cv = sqlx::query!("SELECT id, status::text, unread_count, last_message_at FROM conversations WHERE id = $1 AND organization_id = $2", id, org_id)
        .fetch_optional(&state.db).await.map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Conversation".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "id": cv.id, "status": cv.status, "unread_count": cv.unread_count }))))
}

async fn get_messages(State(state): State<AppState>, RequireOrgViewer(auth): RequireOrgViewer, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
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

async fn send_message(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Path(id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    // Load conversation — verify org ownership
    let conv = sqlx::query!(
        r#"
        SELECT cv.id, cv.wa_account_id, cv.contact_id, cv.status::text
        FROM conversations cv
        WHERE cv.id = $1 AND cv.organization_id = $2
        "#,
        id, org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Conversation".into()))?;

    // Load WA account credentials
    let wa_account = sqlx::query!(
        "SELECT phone_number_id, access_token_enc FROM whatsapp_accounts WHERE id = $1",
        conv.wa_account_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or(AppError::WaNotConnected)?;

    // Load contact phone number
    let contact = sqlx::query!(
        "SELECT phone_number FROM contacts WHERE id = $1",
        conv.contact_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Contact".into()))?;

    let access_token = match wa_account.access_token_enc {
        Some(enc) => {
            let key = &state.config.encryption_key;
            crate::utils::encryption::decrypt(&enc, key).unwrap_or_default()
        }
        None => String::new(),
    };

    let body_text = req.body
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("message body is required".into()))?
        .to_string();

    // Send via WhatsApp API
    let wa_service = WhatsAppService::new(&state);
    let send_resp = wa_service
        .send_text(
            wa_account.phone_number_id.as_deref().unwrap_or_default(),
            &access_token,
            &contact.phone_number,
            &body_text,
        )
        .await?;

    let wa_message_id = send_resp.messages.first().map(|m| m.id.clone());

    // Persist message row
    let message_id = sqlx::query_scalar!(
        r#"
        INSERT INTO messages (
            organization_id, wa_account_id, contact_id, conversation_id,
            wa_message_id, direction, type, body, status
        ) VALUES ($1, $2, $3, $4, $5, 'outbound', 'text', $6, 'sent')
        RETURNING id
        "#,
        org_id,
        conv.wa_account_id,
        conv.contact_id,
        id,
        wa_message_id.as_deref(),
        body_text
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Update conversation metadata
    sqlx::query!(
        r#"
        UPDATE conversations
        SET last_message_at   = NOW(),
            last_message_body = $1,
            last_message_dir  = 'outbound'
        WHERE id = $2
        "#,
        body_text,
        id
    )
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({
            "message_id": message_id,
            "wa_message_id": wa_message_id,
            "status": "sent"
        }))),
    ))
}

async fn assign_conversation(State(state): State<AppState>, RequireOrgMember(auth): RequireOrgMember, Path(id): Path<Uuid>, Json(req): Json<crate::models::conversation::AssignConversationRequest>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE conversations SET assigned_to = $1, assigned_at = NOW() WHERE id = $2 AND organization_id = $3", req.user_id, id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Assigned")))
}

async fn resolve_conversation(State(state): State<AppState>, RequireOrgMember(auth): RequireOrgMember, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE conversations SET status = 'resolved', resolved_at = NOW() WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Resolved")))
}

async fn reopen_conversation(State(state): State<AppState>, RequireOrgMember(auth): RequireOrgMember, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!("UPDATE conversations SET status = 'open', resolved_at = NULL WHERE id = $1 AND organization_id = $2", id, org_id)
        .execute(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Reopened")))
}
