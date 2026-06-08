use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::pagination::ApiResponse,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/overview", get(get_overview))
        .route("/campaigns", get(get_campaign_analytics))
        .route("/messages", get(get_message_analytics))
        .route("/contacts", get(get_contact_analytics))
        .with_state(state)
}

#[derive(Deserialize)]
struct AnalyticsQuery {
    from: Option<String>,    // ISO date string
    to: Option<String>,
    wa_account_id: Option<uuid::Uuid>,
    granularity: Option<String>,  // hour, day, week, month
}

async fn get_overview(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    // Overview: total messages sent, delivered, read, failed; campaigns run; contacts; revenue
    let data = sqlx::query!(
        r#"
        SELECT
            COUNT(DISTINCT c.id) FILTER (WHERE c.status = 'completed') as campaigns_completed,
            COUNT(DISTINCT c.id) FILTER (WHERE c.status = 'running') as campaigns_running,
            COALESCE(SUM(c.sent_count), 0) as total_sent,
            COALESCE(SUM(c.delivered_count), 0) as total_delivered,
            COALESCE(SUM(c.read_count), 0) as total_read,
            COALESCE(SUM(c.failed_count), 0) as total_failed,
            COUNT(DISTINCT ct.id) as total_contacts,
            COUNT(DISTINCT ct.id) FILTER (WHERE ct.wa_status = 'active') as active_contacts
        FROM campaigns c
        CROSS JOIN contacts ct
        WHERE c.organization_id = $1
          AND ct.organization_id = $1
          AND c.deleted_at IS NULL
          AND ct.deleted_at IS NULL
        "#,
        org_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "campaigns": {
            "completed": data.campaigns_completed.unwrap_or(0),
            "running": data.campaigns_running.unwrap_or(0),
        },
        "messages": {
            "total_sent": data.total_sent.unwrap_or(0),
            "total_delivered": data.total_delivered.unwrap_or(0),
            "total_read": data.total_read.unwrap_or(0),
            "total_failed": data.total_failed.unwrap_or(0),
            "delivery_rate": if data.total_sent.unwrap_or(0) > 0 {
                (data.total_delivered.unwrap_or(0) as f64 / data.total_sent.unwrap_or(1) as f64) * 100.0
            } else { 0.0 },
            "read_rate": if data.total_sent.unwrap_or(0) > 0 {
                (data.total_read.unwrap_or(0) as f64 / data.total_sent.unwrap_or(1) as f64) * 100.0
            } else { 0.0 },
        },
        "contacts": {
            "total": data.total_contacts.unwrap_or(0),
            "active": data.active_contacts.unwrap_or(0),
        }
    }))))
}

async fn get_campaign_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    let campaigns = sqlx::query!(
        r#"
        SELECT
            id, name, status::text, type::text,
            total_recipients, sent_count, delivered_count, read_count, failed_count,
            created_at, started_at, completed_at
        FROM campaigns
        WHERE organization_id = $1
          AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = campaigns.iter().map(|c| {
        let delivery_rate = if c.sent_count > 0 {
            (c.delivered_count as f64 / c.sent_count as f64) * 100.0
        } else { 0.0 };

        serde_json::json!({
            "id": c.id,
            "name": c.name,
            "status": c.status,
            "type": c.r#type,
            "total_recipients": c.total_recipients,
            "sent_count": c.sent_count,
            "delivered_count": c.delivered_count,
            "read_count": c.read_count,
            "failed_count": c.failed_count,
            "delivery_rate": delivery_rate,
            "created_at": c.created_at,
            "completed_at": c.completed_at,
        })
    }).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "campaigns": data }))))
}

async fn get_message_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(_query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    // Message volume timeseries (last 30 days, daily)
    let timeseries = sqlx::query!(
        r#"
        SELECT
            DATE_TRUNC('day', created_at) as date,
            COUNT(*) FILTER (WHERE direction = 'outbound') as sent,
            COUNT(*) FILTER (WHERE status = 'delivered') as delivered,
            COUNT(*) FILTER (WHERE status = 'read') as read,
            COUNT(*) FILTER (WHERE direction = 'inbound') as received
        FROM messages
        WHERE organization_id = $1
          AND created_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE_TRUNC('day', created_at)
        ORDER BY date ASC
        "#,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<serde_json::Value> = timeseries.iter().map(|row| {
        serde_json::json!({
            "date": row.date,
            "sent": row.sent.unwrap_or(0),
            "delivered": row.delivered.unwrap_or(0),
            "read": row.read.unwrap_or(0),
            "received": row.received.unwrap_or(0),
        })
    }).collect();

    Ok(Json(ApiResponse::ok(serde_json::json!({ "timeseries": data }))))
}

async fn get_contact_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(_query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    let stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE wa_status = 'active') as active,
            COUNT(*) FILTER (WHERE wa_status = 'unsubscribed') as unsubscribed,
            COUNT(*) FILTER (WHERE wa_status = 'blocked') as blocked,
            COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days') as added_this_month,
            COUNT(*) FILTER (WHERE source = 'csv_import') as from_import,
            COUNT(*) FILTER (WHERE source = 'manual') as from_manual,
            COUNT(*) FILTER (WHERE source = 'whatsapp_inbound') as from_inbound
        FROM contacts
        WHERE organization_id = $1 AND deleted_at IS NULL
        "#,
        org_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "total": stats.total.unwrap_or(0),
        "active": stats.active.unwrap_or(0),
        "unsubscribed": stats.unsubscribed.unwrap_or(0),
        "blocked": stats.blocked.unwrap_or(0),
        "added_this_month": stats.added_this_month.unwrap_or(0),
        "by_source": {
            "import": stats.from_import.unwrap_or(0),
            "manual": stats.from_manual.unwrap_or(0),
            "inbound": stats.from_inbound.unwrap_or(0),
        }
    }))))
}
