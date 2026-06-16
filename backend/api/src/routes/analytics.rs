use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::RequireOrgViewer,
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
    from: Option<String>,          // ISO 8601 date string
    to: Option<String>,
    wa_account_id: Option<uuid::Uuid>,
    granularity: Option<String>,   // day | week | month
}

fn parse_date(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .map(|d| {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                        d.and_hms_opt(0, 0, 0).unwrap(),
                        chrono::Utc,
                    )
                })
        })
}

// ── GET /analytics/overview ──────────────────────────────────────────────────

async fn get_overview(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    let from = query
        .from
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query
        .to
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(chrono::Utc::now);

    // Campaigns aggregate (independent query — no CROSS JOIN)
    let campaign_data = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'completed') as campaigns_completed,
            COUNT(*) FILTER (WHERE status = 'running')   as campaigns_running,
            COALESCE(SUM(sent_count),      0) as total_sent,
            COALESCE(SUM(delivered_count), 0) as total_delivered,
            COALESCE(SUM(read_count),      0) as total_read,
            COALESCE(SUM(failed_count),    0) as total_failed
        FROM campaigns
        WHERE organization_id = $1
          AND deleted_at IS NULL
          AND created_at BETWEEN $2 AND $3
        "#,
        org_id, from, to
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Contacts aggregate (separate query)
    let contact_data = sqlx::query!(
        r#"
        SELECT
            COUNT(*)                                          as total_contacts,
            COUNT(*) FILTER (WHERE wa_status = 'active')     as active_contacts
        FROM contacts
        WHERE organization_id = $1 AND deleted_at IS NULL
        "#,
        org_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let total_sent = campaign_data.total_sent.unwrap_or(0);
    let total_delivered = campaign_data.total_delivered.unwrap_or(0);
    let total_read = campaign_data.total_read.unwrap_or(0);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "date_range": { "from": from, "to": to },
        "campaigns": {
            "completed": campaign_data.campaigns_completed.unwrap_or(0),
            "running":   campaign_data.campaigns_running.unwrap_or(0),
        },
        "messages": {
            "total_sent":      total_sent,
            "total_delivered": total_delivered,
            "total_read":      total_read,
            "total_failed":    campaign_data.total_failed.unwrap_or(0),
            "delivery_rate": if total_sent > 0 {
                (total_delivered as f64 / total_sent as f64) * 100.0
            } else { 0.0 },
            "read_rate": if total_sent > 0 {
                (total_read as f64 / total_sent as f64) * 100.0
            } else { 0.0 },
        },
        "contacts": {
            "total":  contact_data.total_contacts.unwrap_or(0),
            "active": contact_data.active_contacts.unwrap_or(0),
        }
    }))))
}

// ── GET /analytics/campaigns ─────────────────────────────────────────────────

async fn get_campaign_analytics(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    let from = query
        .from
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(90));
    let to = query
        .to
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(chrono::Utc::now);

    let campaigns = sqlx::query!(
        r#"
        SELECT
            id, name, status::text, type::text,
            total_recipients, sent_count, delivered_count, read_count, failed_count,
            created_at, started_at, completed_at
        FROM campaigns
        WHERE organization_id = $1
          AND deleted_at IS NULL
          AND created_at BETWEEN $2 AND $3
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        org_id, from, to
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

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "date_range": { "from": from, "to": to },
        "campaigns": data
    }))))
}

// ── GET /analytics/messages ──────────────────────────────────────────────────

async fn get_message_analytics(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    let from = query
        .from
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query
        .to
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(chrono::Utc::now);

    // Granularity: day (default), week, month
    let trunc = match query.granularity.as_deref().unwrap_or("day") {
        "week"  => "week",
        "month" => "month",
        _       => "day",
    };

    // We build the granularity into the query via a raw string since sqlx
    // doesn't support parameterized DATE_TRUNC levels.
    let timeseries = match trunc {
        "week" => {
            sqlx::query!(
                r#"
                SELECT
                    DATE_TRUNC('week', created_at) as date,
                    COUNT(*) FILTER (WHERE direction = 'outbound') as sent,
                    COUNT(*) FILTER (WHERE status = 'delivered')   as delivered,
                    COUNT(*) FILTER (WHERE status = 'read')        as read,
                    COUNT(*) FILTER (WHERE direction = 'inbound')  as received
                FROM messages
                WHERE organization_id = $1
                  AND created_at BETWEEN $2 AND $3
                GROUP BY DATE_TRUNC('week', created_at)
                ORDER BY date ASC
                "#,
                org_id, from, to
            )
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|r| serde_json::json!({
                "date": r.date,
                "sent": r.sent.unwrap_or(0),
                "delivered": r.delivered.unwrap_or(0),
                "read": r.read.unwrap_or(0),
                "received": r.received.unwrap_or(0),
            }))
            .collect::<Vec<_>>()
        }
        "month" => {
            sqlx::query!(
                r#"
                SELECT
                    DATE_TRUNC('month', created_at) as date,
                    COUNT(*) FILTER (WHERE direction = 'outbound') as sent,
                    COUNT(*) FILTER (WHERE status = 'delivered')   as delivered,
                    COUNT(*) FILTER (WHERE status = 'read')        as read,
                    COUNT(*) FILTER (WHERE direction = 'inbound')  as received
                FROM messages
                WHERE organization_id = $1
                  AND created_at BETWEEN $2 AND $3
                GROUP BY DATE_TRUNC('month', created_at)
                ORDER BY date ASC
                "#,
                org_id, from, to
            )
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|r| serde_json::json!({
                "date": r.date,
                "sent": r.sent.unwrap_or(0),
                "delivered": r.delivered.unwrap_or(0),
                "read": r.read.unwrap_or(0),
                "received": r.received.unwrap_or(0),
            }))
            .collect::<Vec<_>>()
        }
        _ => {
            // Default: daily
            sqlx::query!(
                r#"
                SELECT
                    DATE_TRUNC('day', created_at) as date,
                    COUNT(*) FILTER (WHERE direction = 'outbound') as sent,
                    COUNT(*) FILTER (WHERE status = 'delivered')   as delivered,
                    COUNT(*) FILTER (WHERE status = 'read')        as read,
                    COUNT(*) FILTER (WHERE direction = 'inbound')  as received
                FROM messages
                WHERE organization_id = $1
                  AND created_at BETWEEN $2 AND $3
                GROUP BY DATE_TRUNC('day', created_at)
                ORDER BY date ASC
                "#,
                org_id, from, to
            )
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|r| serde_json::json!({
                "date": r.date,
                "sent": r.sent.unwrap_or(0),
                "delivered": r.delivered.unwrap_or(0),
                "read": r.read.unwrap_or(0),
                "received": r.received.unwrap_or(0),
            }))
            .collect::<Vec<_>>()
        }
    };

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "date_range": { "from": from, "to": to },
        "granularity": trunc,
        "timeseries": timeseries
    }))))
}

// ── GET /analytics/contacts ──────────────────────────────────────────────────

async fn get_contact_analytics(
    State(state): State<AppState>,
    RequireOrgViewer(auth): RequireOrgViewer,
    Query(query): Query<AnalyticsQuery>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let org_id = auth.org_id.ok_or(AppError::Forbidden)?;

    let from = query
        .from
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));

    let stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*)                                                           as total,
            COUNT(*) FILTER (WHERE wa_status = 'active')                      as active,
            COUNT(*) FILTER (WHERE wa_status = 'unsubscribed')                as unsubscribed,
            COUNT(*) FILTER (WHERE wa_status = 'blocked')                     as blocked,
            COUNT(*) FILTER (WHERE created_at >= $2)                          as added_since,
            COUNT(*) FILTER (WHERE source = 'csv_import')                     as from_import,
            COUNT(*) FILTER (WHERE source = 'manual')                         as from_manual,
            COUNT(*) FILTER (WHERE source = 'whatsapp_inbound')               as from_inbound
        FROM contacts
        WHERE organization_id = $1 AND deleted_at IS NULL
        "#,
        org_id, from
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "total":        stats.total.unwrap_or(0),
        "active":       stats.active.unwrap_or(0),
        "unsubscribed": stats.unsubscribed.unwrap_or(0),
        "blocked":      stats.blocked.unwrap_or(0),
        "added_since":  stats.added_since.unwrap_or(0),
        "by_source": {
            "import":  stats.from_import.unwrap_or(0),
            "manual":  stats.from_manual.unwrap_or(0),
            "inbound": stats.from_inbound.unwrap_or(0),
        }
    }))))
}
