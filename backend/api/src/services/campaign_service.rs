use uuid::Uuid;
use crate::{
    errors::{AppError, AppResult},
    models::campaign::{
        Campaign, CampaignListQuery, CampaignStats, CreateCampaignRequest,
        CampaignStatus, ScheduleCampaignRequest, UpdateCampaignRequest,
    },
    AppState,
};

pub struct CampaignService<'a> {
    state: &'a AppState,
}

impl<'a> CampaignService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn list_campaigns(
        &self,
        org_id: Uuid,
        query: CampaignListQuery,
    ) -> AppResult<serde_json::Value> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).min(100);
        let offset = ((page - 1) * limit) as i64;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM campaigns WHERE organization_id = $1 AND deleted_at IS NULL",
            org_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .unwrap_or(0);

        let campaigns = sqlx::query!(
            r#"
            SELECT id, name, description, type::text, status::text,
                   total_recipients, sent_count, delivered_count, read_count, failed_count,
                   scheduled_at, started_at, completed_at, created_at
            FROM campaigns
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            org_id,
            limit as i64,
            offset
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let data: Vec<serde_json::Value> = campaigns.iter().map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "type": c.r#type,
                "status": c.status,
                "total_recipients": c.total_recipients,
                "sent_count": c.sent_count,
                "delivered_count": c.delivered_count,
                "read_count": c.read_count,
                "failed_count": c.failed_count,
                "delivery_rate": if c.sent_count > 0 {
                    (c.delivered_count as f64 / c.sent_count as f64) * 100.0
                } else { 0.0 },
                "scheduled_at": c.scheduled_at,
                "started_at": c.started_at,
                "completed_at": c.completed_at,
                "created_at": c.created_at,
            })
        }).collect();

        Ok(serde_json::json!({
            "data": data,
            "pagination": {
                "total": total,
                "page": page,
                "limit": limit,
                "total_pages": ((total as f64) / (limit as f64)).ceil() as u32
            }
        }))
    }

    pub async fn create_campaign(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        req: CreateCampaignRequest,
    ) -> AppResult<serde_json::Value> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO campaigns (
                organization_id, wa_account_id, name, description, type, status,
                target_type, target_group_id, target_segment_id, target_contact_ids,
                template_id, message_body, media_url, media_type, buttons,
                send_rate, timezone, created_by
            )
            VALUES ($1, $2, $3, $4, $5::campaign_type, 'draft', $6::campaign_target_type,
                    $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING id
            "#,
            org_id,
            req.wa_account_id,
            req.name,
            req.description,
            req.r#type as _,
            req.target_type as _,
            req.target_group_id,
            req.target_segment_id,
            req.target_contact_ids.as_deref(),
            req.template_id,
            req.message_body,
            req.media_url,
            req.media_type,
            req.buttons,
            req.send_rate.unwrap_or(60),
            req.timezone.as_deref().unwrap_or("Asia/Kolkata"),
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        self.get_campaign(org_id, id).await
    }

    pub async fn get_campaign(&self, org_id: Uuid, id: Uuid) -> AppResult<serde_json::Value> {
        let c = sqlx::query!(
            r#"
            SELECT id, name, description, type::text, status::text, wa_account_id,
                   target_type::text, target_group_id, target_segment_id,
                   template_id, message_body, media_url, scheduled_at, send_rate, timezone,
                   total_recipients, sent_count, delivered_count, read_count, failed_count, reply_count,
                   started_at, completed_at, created_at, updated_at
            FROM campaigns
            WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL
            "#,
            id, org_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Campaign".into()))?;

        let delivery_rate = if c.sent_count > 0 {
            (c.delivered_count as f64 / c.sent_count as f64) * 100.0
        } else { 0.0 };

        Ok(serde_json::json!({
            "id": c.id, "name": c.name, "description": c.description,
            "type": c.r#type, "status": c.status,
            "wa_account_id": c.wa_account_id,
            "target_type": c.target_type,
            "message_body": c.message_body,
            "template_id": c.template_id,
            "send_rate": c.send_rate, "timezone": c.timezone,
            "stats": {
                "total_recipients": c.total_recipients,
                "sent": c.sent_count, "delivered": c.delivered_count,
                "read": c.read_count, "failed": c.failed_count, "replies": c.reply_count,
                "delivery_rate": delivery_rate,
            },
            "scheduled_at": c.scheduled_at,
            "started_at": c.started_at, "completed_at": c.completed_at,
            "created_at": c.created_at, "updated_at": c.updated_at,
        }))
    }

    pub async fn update_campaign(
        &self,
        org_id: Uuid,
        id: Uuid,
        req: UpdateCampaignRequest,
    ) -> AppResult<serde_json::Value> {
        // Verify ownership and draft status
        let status = sqlx::query_scalar!(
            "SELECT status::text FROM campaigns WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
            id, org_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Campaign".into()))?;

        if status.as_deref() != Some("draft") {
            return Err(AppError::InvalidCampaignState);
        }

        sqlx::query!(
            r#"
            UPDATE campaigns
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                message_body = COALESCE($3, message_body),
                template_id = COALESCE($4, template_id),
                send_rate = COALESCE($5, send_rate)
            WHERE id = $6 AND organization_id = $7
            "#,
            req.name, req.description, req.message_body, req.template_id,
            req.send_rate, id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        self.get_campaign(org_id, id).await
    }

    pub async fn delete_campaign(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        sqlx::query!(
            "UPDATE campaigns SET deleted_at = NOW() WHERE id = $1 AND organization_id = $2",
            id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub async fn launch_campaign(
        &self,
        org_id: Uuid,
        id: Uuid,
        user_id: Uuid,
    ) -> AppResult<serde_json::Value> {
        // Transition to running
        sqlx::query!(
            r#"
            UPDATE campaigns
            SET status = 'running', started_at = NOW()
            WHERE id = $1 AND organization_id = $2 AND status IN ('draft', 'scheduled', 'paused')
            "#,
            id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        // Trigger n8n event
        let n8n = crate::services::n8n_service::N8nService::new(self.state);
        n8n.fire_event(org_id, crate::services::n8n_service::N8nEvent::CampaignLaunched {
            campaign_id: id, org_id,
        }).await;

        self.get_campaign(org_id, id).await
    }

    pub async fn schedule_campaign(
        &self,
        org_id: Uuid,
        id: Uuid,
        req: ScheduleCampaignRequest,
    ) -> AppResult<serde_json::Value> {
        sqlx::query!(
            "UPDATE campaigns SET status = 'scheduled', scheduled_at = $1 WHERE id = $2 AND organization_id = $3 AND status = 'draft'",
            req.scheduled_at, id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        self.get_campaign(org_id, id).await
    }

    pub async fn pause_campaign(&self, org_id: Uuid, id: Uuid) -> AppResult<serde_json::Value> {
        sqlx::query!(
            "UPDATE campaigns SET status = 'paused' WHERE id = $1 AND organization_id = $2 AND status = 'running'",
            id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        self.get_campaign(org_id, id).await
    }

    pub async fn resume_campaign(&self, org_id: Uuid, id: Uuid) -> AppResult<serde_json::Value> {
        sqlx::query!(
            "UPDATE campaigns SET status = 'running' WHERE id = $1 AND organization_id = $2 AND status = 'paused'",
            id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        self.get_campaign(org_id, id).await
    }

    pub async fn cancel_campaign(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        sqlx::query!(
            "UPDATE campaigns SET status = 'cancelled' WHERE id = $1 AND organization_id = $2",
            id, org_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub async fn clone_campaign(
        &self,
        org_id: Uuid,
        id: Uuid,
        user_id: Uuid,
    ) -> AppResult<serde_json::Value> {
        let new_id = sqlx::query_scalar!(
            r#"
            INSERT INTO campaigns (
                organization_id, wa_account_id, name, description, type, status,
                target_type, target_group_id, target_segment_id, template_id,
                message_body, media_url, media_type, buttons, send_rate, timezone, created_by
            )
            SELECT organization_id, wa_account_id, name || ' (Copy)', description, type, 'draft',
                   target_type, target_group_id, target_segment_id, template_id,
                   message_body, media_url, media_type, buttons, send_rate, timezone, $3
            FROM campaigns WHERE id = $1 AND organization_id = $2
            RETURNING id
            "#,
            id, org_id, user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        self.get_campaign(org_id, new_id).await
    }

    pub async fn get_campaign_stats(&self, org_id: Uuid, id: Uuid) -> AppResult<serde_json::Value> {
        self.get_campaign(org_id, id).await
    }

    pub async fn get_campaign_messages(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        query: serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        let messages = sqlx::query!(
            r#"
            SELECT m.id, m.wa_message_id, m.status::text, m.direction::text,
                   m.body, m.sent_at, m.delivered_at, m.read_at, m.failed_at,
                   c.phone_number, c.first_name, c.last_name
            FROM messages m
            LEFT JOIN contacts c ON c.id = m.contact_id
            WHERE m.campaign_id = $1 AND m.organization_id = $2
            ORDER BY m.created_at DESC
            LIMIT 100
            "#,
            campaign_id, org_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let data: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({
                "id": m.id,
                "status": m.status,
                "contact": {
                    "phone": m.phone_number,
                    "name": format!("{} {}", m.first_name.as_deref().unwrap_or(""), m.last_name.as_deref().unwrap_or("")).trim().to_string(),
                },
                "sent_at": m.sent_at,
                "delivered_at": m.delivered_at,
                "read_at": m.read_at,
            })
        }).collect();

        Ok(serde_json::json!({ "messages": data }))
    }
}
