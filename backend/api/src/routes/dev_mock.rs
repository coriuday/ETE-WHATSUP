use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::RequireOrgMember,
    models::{message::MessageStatus, pagination::ApiResponse},
    providers::{
        dispatch::ensure_mock_account,
        lifecycle::transition_message,
    },
    services::{automation_engine, n8n_service::{N8nEvent, N8nService}},
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/inbound", post(simulate_inbound))
        .route("/status", post(simulate_status))
        .with_state(state)
}

#[derive(Deserialize)]
struct InboundBody {
    phone_number: String,
    body: String,
    contact_name: Option<String>,
}

#[derive(Deserialize)]
struct StatusBody {
    message_id: Uuid,
    status: String,
}

async fn simulate_inbound(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Json(req): Json<InboundBody>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    if !state.config.enable_mock_provider {
        return Err(AppError::Forbidden);
    }
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let account = ensure_mock_account(&state, org_id).await?;

    let first = req.contact_name.clone();
    let contact_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO contacts (organization_id, phone_number, first_name, wa_status, source)
        VALUES ($1, $2, $3, 'active', 'whatsapp_inbound')
        ON CONFLICT (organization_id, phone_number)
        DO UPDATE SET first_name = COALESCE(EXCLUDED.first_name, contacts.first_name)
        RETURNING id
        "#,
    )
    .bind(org_id)
    .bind(&req.phone_number)
    .bind(&first)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let conversation_id: Uuid = if let Some(existing) = sqlx::query_scalar::<_, Uuid>(
        r#"SELECT id FROM conversations WHERE organization_id = $1 AND contact_id = $2 AND wa_account_id = $3 LIMIT 1"#,
    )
    .bind(org_id)
    .bind(contact_id)
    .bind(account.id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    {
        sqlx::query(
            r#"UPDATE conversations SET status = 'open', last_message_at = NOW(), last_message_body = $2,
               last_message_dir = 'inbound', unread_count = unread_count + 1 WHERE id = $1"#,
        )
        .bind(existing)
        .bind(&req.body)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO conversations (organization_id, wa_account_id, contact_id, status, last_message_at, last_message_body, last_message_dir, unread_count, is_in_session, session_expires_at)
            VALUES ($1, $2, $3, 'open', NOW(), $4, 'inbound', 1, TRUE, NOW() + INTERVAL '24 hours')
            RETURNING id
            "#,
        )
        .bind(org_id)
        .bind(account.id)
        .bind(contact_id)
        .bind(&req.body)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?
    };

    let message_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO messages (
            organization_id, wa_account_id, contact_id, conversation_id,
            wa_message_id, direction, type, body, status
        ) VALUES ($1, $2, $3, $4, $5, 'inbound', 'text', $6, 'delivered')
        RETURNING id
        "#,
    )
    .bind(org_id)
    .bind(account.id)
    .bind(contact_id)
    .bind(conversation_id)
    .bind(format!("mock:in:{}", Uuid::new_v4()))
    .bind(&req.body)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    N8nService::new(&state)
        .fire_event(
            org_id,
            N8nEvent::MessageReceived {
                contact_id,
                org_id,
                message_id,
            },
        )
        .await;
    automation_engine::dispatch(
        &state,
        org_id,
        "message.received",
        serde_json::json!({
            "contact_id": contact_id,
            "conversation_id": conversation_id,
            "message_id": message_id,
            "body": req.body
        }),
    )
    .await;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "conversation_id": conversation_id,
        "contact_id": contact_id,
        "message_id": message_id
    }))))
}

async fn simulate_status(
    State(state): State<AppState>,
    RequireOrgMember(auth): RequireOrgMember,
    Json(req): Json<StatusBody>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    if !state.config.enable_mock_provider {
        return Err(AppError::Forbidden);
    }
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let to = match req.status.as_str() {
        "delivered" => MessageStatus::Delivered,
        "read" => MessageStatus::Read,
        "failed" => MessageStatus::Failed,
        "sent" => MessageStatus::Sent,
        "processing" => MessageStatus::Processing,
        "retrying" => MessageStatus::Retrying,
        "cancelled" => MessageStatus::Cancelled,
        other => return Err(AppError::Validation(format!("unsupported status {other}"))),
    };
    let status = transition_message(&state.db, org_id, req.message_id, to, Some("mock simulation")).await?;

    if matches!(to, MessageStatus::Delivered | MessageStatus::Read | MessageStatus::Failed) {
        let _ = sqlx::query(
            r#"
            UPDATE campaigns c
            SET delivered_count = CASE WHEN $2 = 'delivered' THEN delivered_count + 1 ELSE delivered_count END,
                read_count = CASE WHEN $2 = 'read' THEN read_count + 1 ELSE read_count END,
                failed_count = CASE WHEN $2 = 'failed' THEN failed_count + 1 ELSE failed_count END
            FROM messages m
            WHERE m.id = $1 AND m.campaign_id = c.id AND m.organization_id = $3
            "#,
        )
        .bind(req.message_id)
        .bind(req.status.as_str())
        .bind(org_id)
        .execute(&state.db)
        .await;
    }

    Ok(Json(ApiResponse::ok(serde_json::json!({ "status": status.as_str() }))))
}
