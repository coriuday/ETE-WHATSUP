use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::RequireOrgViewer,
    models::pagination::{ApiResponse, PaginatedResponse, PaginationQuery},
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(list_messages))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct MessageListQuery {
    page: Option<u32>,
    limit: Option<u32>,
    campaign_id: Option<Uuid>,
    status: Option<String>,
}

async fn list_messages(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<MessageListQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<Value>>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;
    let pagination = PaginationQuery {
        page: query.page,
        limit: query.limit,
    };

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM messages
        WHERE organization_id = $1
          AND ($2::uuid IS NULL OR campaign_id = $2)
          AND ($3::text IS NULL OR status::text = $3)
        "#,
    )
    .bind(org_id)
    .bind(query.campaign_id)
    .bind(&query.status)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?
    .unwrap_or(0);

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            Option<Uuid>,
            Option<Uuid>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT id, campaign_id, contact_id, wa_message_id, direction::text, type::text,
               body, status::text, sent_at, delivered_at, read_at, failed_at, created_at
        FROM messages
        WHERE organization_id = $1
          AND ($2::uuid IS NULL OR campaign_id = $2)
          AND ($3::text IS NULL OR status::text = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(org_id)
    .bind(query.campaign_id)
    .bind(&query.status)
    .bind(pagination.limit_i64())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data = rows
        .into_iter()
        .map(
            |(
                id,
                campaign_id,
                contact_id,
                wa_message_id,
                direction,
                msg_type,
                body,
                status,
                sent_at,
                delivered_at,
                read_at,
                failed_at,
                created_at,
            )| {
                serde_json::json!({
                    "id": id,
                    "campaign_id": campaign_id,
                    "contact_id": contact_id,
                    "wa_message_id": wa_message_id,
                    "direction": direction,
                    "type": msg_type,
                    "body": body,
                    "status": status,
                    "sent_at": sent_at,
                    "delivered_at": delivered_at,
                    "read_at": read_at,
                    "failed_at": failed_at,
                    "created_at": created_at,
                })
            },
        )
        .collect();

    Ok(Json(ApiResponse::ok(PaginatedResponse::new(
        data, total, &pagination,
    ))))
}
