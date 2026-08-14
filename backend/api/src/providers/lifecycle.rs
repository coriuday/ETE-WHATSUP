use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::message::MessageStatus;

const ALLOWED: &[(&str, &str)] = &[
    ("queued", "processing"),
    ("queued", "cancelled"),
    ("processing", "sent"),
    ("processing", "failed"),
    ("processing", "retrying"),
    ("processing", "cancelled"),
    ("sending", "sent"),
    ("sending", "failed"),
    ("sending", "retrying"),
    ("retrying", "processing"),
    ("retrying", "failed"),
    ("retrying", "cancelled"),
    ("sent", "delivered"),
    ("sent", "failed"),
    ("sent", "read"),
    ("delivered", "read"),
    ("delivered", "failed"),
];

pub fn can_transition(from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }
    ALLOWED.iter().any(|(a, b)| *a == from && *b == to)
}

fn timestamp_column(to: &str) -> Option<&'static str> {
    match to {
        "sent" => Some("sent_at"),
        "delivered" => Some("delivered_at"),
        "read" => Some("read_at"),
        "failed" | "cancelled" => Some("failed_at"),
        _ => None,
    }
}

pub async fn transition_message(
    db: &PgPool,
    organization_id: Uuid,
    message_id: Uuid,
    to: MessageStatus,
    reason: Option<&str>,
) -> AppResult<MessageStatus> {
    let row = sqlx::query("SELECT status::text as status FROM messages WHERE id = $1 AND organization_id = $2")
        .bind(message_id)
        .bind(organization_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound("Message".into()))?;

    let from = row
        .try_get::<Option<String>, _>("status")
        .ok()
        .flatten()
        .unwrap_or_else(|| "queued".into());
    let to_str = to.as_str();
    if !can_transition(&from, to_str) {
        return Err(AppError::BadRequest(format!(
            "illegal message transition {from} -> {to_str}"
        )));
    }

    let ts_col = timestamp_column(to_str);
    if let Some(col) = ts_col {
        let sql = format!(
            "UPDATE messages SET status = $1::{}, {} = COALESCE({}, NOW()) WHERE id = $2",
            "message_status", col, col
        );
        // Dynamic column names are constrained to a known allow-list above.
        sqlx::query(&sql)
            .bind(to_str)
            .bind(message_id)
            .execute(db)
            .await?;
    } else {
        sqlx::query("UPDATE messages SET status = $1::message_status WHERE id = $2")
            .bind(to_str)
            .bind(message_id)
            .execute(db)
            .await?;
    }

    let _ = sqlx::query(
        r#"
        INSERT INTO message_status_events (message_id, organization_id, from_status, to_status, reason)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(message_id)
    .bind(organization_id)
    .bind(&from)
    .bind(to_str)
    .bind(reason)
    .execute(db)
    .await;

    let _ = Utc::now();
    Ok(to)
}

#[cfg(test)]
mod tests {
    use super::can_transition;

    #[test]
    fn allows_sent_to_delivered_to_read() {
        assert!(can_transition("queued", "processing"));
        assert!(can_transition("processing", "sent"));
        assert!(can_transition("sent", "delivered"));
        assert!(can_transition("delivered", "read"));
        assert!(!can_transition("read", "queued"));
        assert!(!can_transition("failed", "sent"));
    }
}

pub async fn insert_outbound_sent(
    db: &PgPool,
    organization_id: Uuid,
    wa_account_id: Uuid,
    campaign_id: Option<Uuid>,
    contact_id: Uuid,
    conversation_id: Option<Uuid>,
    provider_message_id: &str,
    body: &str,
    msg_type: &str,
) -> AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO messages (
            organization_id, wa_account_id, campaign_id, contact_id, conversation_id,
            wa_message_id, direction, type, body, status, sent_at
        ) VALUES ($1, $2, $3, $4, $5, $6, 'outbound', $7::message_type, $8, 'sent', NOW())
        RETURNING id
        "#,
    )
    .bind(organization_id)
    .bind(wa_account_id)
    .bind(campaign_id)
    .bind(contact_id)
    .bind(conversation_id)
    .bind(provider_message_id)
    .bind(msg_type)
    .bind(body)
    .fetch_one(db)
    .await?;

    let _ = sqlx::query(
        r#"
        INSERT INTO message_status_events (message_id, organization_id, from_status, to_status, reason)
        VALUES ($1, $2, 'queued', 'sent', 'provider accepted')
        "#,
    )
    .bind(id)
    .bind(organization_id)
    .execute(db)
    .await;

    Ok(id)
}
