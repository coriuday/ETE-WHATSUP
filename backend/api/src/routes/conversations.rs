use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::{RequireOrgMember, RequireOrgViewer},
    models::{conversation::SendMessageRequest, pagination::ApiResponse},
    providers::{
        dispatch::{load_account, send_media, send_template, send_text},
        lifecycle::insert_outbound_sent,
        OutboundMedia, OutboundTemplate, OutboundText,
    },
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
        .route("/:id/notes", get(list_notes).post(add_note))
        .with_state(state)
}

async fn list_conversations(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<crate::models::conversation::ConversationListQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let status = query.status.map(|s| s.as_str().to_string()).unwrap_or_default();
    let search = query.search.clone().unwrap_or_default();
    let assigned_to = query.assigned_to;
    let unread_only = query.unread_only.unwrap_or(false);
    let unassigned = query.unassigned.unwrap_or(false);
    let mine = query.mine.unwrap_or(false);

    let convs = sqlx::query!(
        r#"SELECT cv.id, cv.status::text, cv.unread_count, cv.last_message_at, cv.last_message_body,
                  cv.assigned_to, c.phone_number, c.first_name, c.last_name, c.id as contact_id, c.tags, c.custom_fields
           FROM conversations cv JOIN contacts c ON c.id = cv.contact_id
           WHERE cv.organization_id = $1
             AND ($2 = '' OR cv.status::text = $2)
             AND ($3 = '' OR c.phone_number ILIKE '%' || $3 || '%' OR coalesce(c.first_name,'') ILIKE '%' || $3 || '%' OR coalesce(cv.last_message_body,'') ILIKE '%' || $3 || '%')
             AND ($4::uuid IS NULL OR cv.assigned_to = $4)
             AND (NOT $5 OR cv.unread_count > 0)
             AND (NOT $6 OR cv.assigned_to IS NULL)
             AND (NOT $7 OR cv.assigned_to = $8)
           ORDER BY cv.last_message_at DESC NULLS LAST
           LIMIT 100"#,
        org_id,
        status,
        search,
        assigned_to,
        unread_only,
        unassigned,
        mine,
        auth.id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = convs
        .iter()
        .map(|cv| {
            serde_json::json!({
                "id": cv.id, "status": cv.status, "unread_count": cv.unread_count,
                "last_message_at": cv.last_message_at, "last_message_body": cv.last_message_body,
                "assigned_to": cv.assigned_to,
                "contact": {
                    "id": cv.contact_id,
                    "phone": cv.phone_number,
                    "name": format!("{} {}", cv.first_name.as_deref().unwrap_or(""), cv.last_name.as_deref().unwrap_or("")).trim().to_string(),
                    "tags": cv.tags,
                    "custom_fields": cv.custom_fields
                }
            })
        })
        .collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "conversations": data }))))
}

async fn get_conversation(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let cv = sqlx::query!(
        r#"SELECT cv.id, cv.status::text, cv.unread_count, cv.last_message_at, cv.assigned_to,
                  cv.contact_id, cv.wa_account_id, c.phone_number, c.first_name, c.last_name, c.email, c.tags, c.custom_fields
           FROM conversations cv JOIN contacts c ON c.id = cv.contact_id
           WHERE cv.id = $1 AND cv.organization_id = $2"#,
        id,
        org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Conversation".into()))?;

    let _ = sqlx::query!(
        "UPDATE conversations SET unread_count = 0 WHERE id = $1 AND organization_id = $2",
        id,
        org_id
    )
    .execute(&state.db)
    .await;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": cv.id, "status": cv.status, "unread_count": 0,
        "assigned_to": cv.assigned_to, "wa_account_id": cv.wa_account_id,
        "contact": {
            "id": cv.contact_id, "phone": cv.phone_number,
            "first_name": cv.first_name, "last_name": cv.last_name, "email": cv.email,
            "tags": cv.tags, "custom_fields": cv.custom_fields
        }
    }))))
}

async fn get_messages(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let msgs = sqlx::query!(
        r#"SELECT id, direction::text, type::text, body, media_url, status::text, created_at, sent_at, delivered_at, read_at
           FROM messages WHERE conversation_id = $1 AND organization_id = $2
           ORDER BY created_at ASC LIMIT 200"#,
        id,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = msgs
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id, "direction": m.direction, "type": m.r#type,
                "body": m.body, "media_url": m.media_url, "status": m.status,
                "created_at": m.created_at, "sent_at": m.sent_at,
                "delivered_at": m.delivered_at, "read_at": m.read_at
            })
        })
        .collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "messages": data }))))
}

async fn send_message(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Path(id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let conv = sqlx::query!(
        r#"
        SELECT cv.id, cv.wa_account_id, cv.contact_id, cv.status::text
        FROM conversations cv
        WHERE cv.id = $1 AND cv.organization_id = $2
        "#,
        id,
        org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Conversation".into()))?;

    let wa_account_id = conv.wa_account_id.ok_or(AppError::WaNotConnected)?;
    let account = load_account(&state, wa_account_id, org_id).await?;
    let contact = sqlx::query!(
        "SELECT phone_number FROM contacts WHERE id = $1 AND organization_id = $2",
        conv.contact_id,
        org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Contact".into()))?;

    let body_text = req.body.clone().unwrap_or_default();
    let send = if req.template_id.is_some() {
        let name = req
            .body
            .clone()
            .unwrap_or_else(|| "hello_world".into());
        send_template(
            &state,
            &account,
            OutboundTemplate {
                to: contact.phone_number.clone(),
                template_name: name.clone(),
                language: "en".into(),
                components: vec![],
            },
        )
        .await?
    } else if let Some(media_url) = req.media_url.clone() {
        send_media(
            &state,
            &account,
            OutboundMedia {
                to: contact.phone_number.clone(),
                media_url,
                caption: req.body.clone(),
                mime_type: None,
            },
        )
        .await?
    } else {
        if body_text.trim().is_empty() {
            return Err(AppError::Validation("message body is required".into()));
        }
        send_text(
            &state,
            &account,
            OutboundText {
                to: contact.phone_number.clone(),
                body: body_text.clone(),
            },
        )
        .await?
    };

    let preview = if body_text.is_empty() {
        send.provider_message_id.clone()
    } else {
        body_text.clone()
    };

    let message_id = insert_outbound_sent(
        &state.db,
        org_id,
        wa_account_id,
        None,
        conv.contact_id,
        Some(id),
        &send.provider_message_id,
        &preview,
        if req.template_id.is_some() {
            "template"
        } else if req.media_url.is_some() {
            "image"
        } else {
            "text"
        },
    )
    .await?;

    sqlx::query!(
        r#"
        UPDATE conversations
        SET last_message_at   = NOW(),
            last_message_body = $1,
            last_message_dir  = 'outbound'
        WHERE id = $2 AND organization_id = $3
        "#,
        preview,
        id,
        org_id
    )
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({
            "message_id": message_id,
            "wa_message_id": send.provider_message_id,
            "status": send.status
        }))),
    ))
}

async fn assign_conversation(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Path(id): Path<Uuid>,
    Json(req): Json<crate::models::conversation::AssignConversationRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!(
        "UPDATE conversations SET assigned_to = $1, assigned_at = NOW() WHERE id = $2 AND organization_id = $3",
        req.user_id,
        id,
        org_id
    )
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Assigned")))
}

async fn resolve_conversation(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!(
        "UPDATE conversations SET status = 'resolved', resolved_at = NOW() WHERE id = $1 AND organization_id = $2",
        id,
        org_id
    )
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Resolved")))
}

async fn reopen_conversation(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    sqlx::query!(
        "UPDATE conversations SET status = 'open', resolved_at = NULL WHERE id = $1 AND organization_id = $2",
        id,
        org_id
    )
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::with_message(serde_json::json!({}), "Reopened")))
}

async fn list_notes(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let notes = sqlx::query!(
        r#"SELECT n.id, n.body, n.created_at, n.created_by
           FROM conversation_notes n
           JOIN conversations c ON c.id = n.conversation_id
           WHERE n.conversation_id = $1 AND c.organization_id = $2
           ORDER BY n.created_at DESC"#,
        id,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "notes": notes.iter().map(|n| serde_json::json!({
        "id": n.id, "body": n.body, "created_at": n.created_at, "created_by": n.created_by
    })).collect::<Vec<_>>() }))))
}

async fn add_note(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let text = body["body"].as_str().unwrap_or("").trim().to_string();
    if text.is_empty() {
        return Err(AppError::Validation("note body is required".into()));
    }
    let exists = sqlx::query_scalar!(
        "SELECT id FROM conversations WHERE id = $1 AND organization_id = $2",
        id,
        org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;
    if exists.is_none() {
        return Err(AppError::NotFound("Conversation".into()));
    }
    let note_id = sqlx::query_scalar!(
        "INSERT INTO conversation_notes (conversation_id, created_by, body) VALUES ($1, $2, $3) RETURNING id",
        id,
        auth.id,
        text
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "id": note_id }))))
}
