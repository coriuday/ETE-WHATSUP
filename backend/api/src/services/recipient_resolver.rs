use uuid::Uuid;
use sqlx::Row;

use crate::{
    errors::{AppError, AppResult},
    AppState,
};

/// A resolved recipient — enough to send a WhatsApp message
#[derive(Debug, Clone)]
pub struct Recipient {
    pub contact_id: Uuid,
    pub phone_number: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Resolve the recipient list for a campaign based on its `target_type`.
///
/// Returns only contacts that are opted-in (`wa_status = 'active'`).
pub async fn resolve_recipients(
    state: &AppState,
    org_id: Uuid,
    campaign_id: Uuid,
) -> AppResult<Vec<Recipient>> {
    // Load campaign target config
    let campaign = sqlx::query!(
        r#"
        SELECT target_type::text, target_group_id, target_segment_id, target_contact_ids
        FROM campaigns
        WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL
        "#,
        campaign_id,
        org_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Campaign".into()))?;

    let recipients = match campaign.target_type.as_deref().unwrap_or("all_contacts") {
        "group" => {
            let group_id = campaign
                .target_group_id
                .ok_or_else(|| AppError::Validation("Campaign missing target_group_id".into()))?;
            resolve_from_group(state, org_id, group_id).await?
        }
        "segment" => {
            let segment_id = campaign
                .target_segment_id
                .ok_or_else(|| AppError::Validation("Campaign missing target_segment_id".into()))?;
            resolve_from_segment(state, org_id, segment_id).await?
        }
        "custom_list" => {
            let ids = campaign
                .target_contact_ids
                .ok_or_else(|| AppError::Validation("Campaign missing target_contact_ids".into()))?;
            resolve_from_list(state, org_id, &ids).await?
        }
        _ => {
            // all_contacts
            resolve_all_contacts(state, org_id).await?
        }
    };

    tracing::info!(
        "Resolved {} recipients for campaign {}",
        recipients.len(),
        campaign_id
    );

    Ok(recipients)
}

async fn resolve_from_group(
    state: &AppState,
    org_id: Uuid,
    group_id: Uuid,
) -> AppResult<Vec<Recipient>> {
    let rows = sqlx::query!(
        r#"
        SELECT c.id, c.phone_number, c.first_name, c.last_name
        FROM contacts c
        JOIN contact_group_members cgm ON cgm.contact_id = c.id
        WHERE cgm.group_id = $1
          AND c.organization_id = $2
          AND c.deleted_at IS NULL
          AND c.wa_status = 'active'
        "#,
        group_id,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| Recipient {
            contact_id: r.id,
            phone_number: r.phone_number,
            first_name: r.first_name,
            last_name: r.last_name,
        })
        .collect())
}

async fn resolve_from_segment(
    state: &AppState,
    org_id: Uuid,
    segment_id: Uuid,
) -> AppResult<Vec<Recipient>> {
    let segment = sqlx::query("SELECT filter_rules FROM contact_segments WHERE id = $1 AND organization_id = $2")
        .bind(segment_id)
        .bind(org_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Segment".into()))?;

    let filter: serde_json::Value = sqlx::Row::try_get(&segment, "filter_rules").unwrap_or(serde_json::json!({}));
    let tag = filter
        .get("tag")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let search = filter
        .get("search")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let rows = sqlx::query(
        r#"
        SELECT c.id, c.phone_number, c.first_name, c.last_name
        FROM contacts c
        WHERE c.organization_id = $1
          AND c.deleted_at IS NULL
          AND c.wa_status = 'active'
          AND ($2 = '' OR $2 = ANY(c.tags))
          AND ($3 = '' OR c.phone_number ILIKE '%' || $3 || '%' OR coalesce(c.first_name,'') ILIKE '%' || $3 || '%')
        "#,
    )
    .bind(org_id)
    .bind(&tag)
    .bind(&search)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            use sqlx::Row;
            Recipient {
            contact_id: r.get("id"),
            phone_number: r.get("phone_number"),
            first_name: r.try_get("first_name").ok().flatten(),
            last_name: r.try_get("last_name").ok().flatten(),
        }})
        .collect())
}

async fn resolve_from_list(
    state: &AppState,
    org_id: Uuid,
    contact_ids: &[Uuid],
) -> AppResult<Vec<Recipient>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, phone_number, first_name, last_name
        FROM contacts
        WHERE id = ANY($1)
          AND organization_id = $2
          AND deleted_at IS NULL
          AND wa_status = 'active'
        "#,
        contact_ids,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| Recipient {
            contact_id: r.id,
            phone_number: r.phone_number,
            first_name: r.first_name,
            last_name: r.last_name,
        })
        .collect())
}

async fn resolve_all_contacts(state: &AppState, org_id: Uuid) -> AppResult<Vec<Recipient>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, phone_number, first_name, last_name
        FROM contacts
        WHERE organization_id = $1
          AND deleted_at IS NULL
          AND wa_status = 'active'
        "#,
        org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| Recipient {
            contact_id: r.id,
            phone_number: r.phone_number,
            first_name: r.first_name,
            last_name: r.last_name,
        })
        .collect())
}
