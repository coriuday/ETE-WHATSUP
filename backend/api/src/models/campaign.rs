use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "campaign_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CampaignType {
    BulkMessage,
    Drip,
    Transactional,
    AbTest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "campaign_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "campaign_target_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CampaignTargetType {
    Group,
    Segment,
    AllContacts,
    CustomList,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub wa_account_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub r#type: CampaignType,
    pub status: CampaignStatus,
    pub target_type: CampaignTargetType,
    pub target_group_id: Option<Uuid>,
    pub target_segment_id: Option<Uuid>,
    pub target_contact_ids: Option<Vec<Uuid>>,
    pub template_id: Option<Uuid>,
    pub message_body: Option<String>,
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub buttons: Option<serde_json::Value>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub send_rate: i32,
    pub timezone: String,
    pub total_recipients: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub read_count: i32,
    pub failed_count: i32,
    pub reply_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Campaign stats summary ───────────────────────────────────
#[derive(Debug, Serialize)]
pub struct CampaignStats {
    pub total_recipients: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub read_count: i32,
    pub failed_count: i32,
    pub reply_count: i32,
    pub delivery_rate: f64,
    pub read_rate: f64,
    pub reply_rate: f64,
}

impl From<&Campaign> for CampaignStats {
    fn from(c: &Campaign) -> Self {
        let total = c.total_recipients as f64;
        let delivery_rate = if total > 0.0 { (c.delivered_count as f64 / total) * 100.0 } else { 0.0 };
        let read_rate = if total > 0.0 { (c.read_count as f64 / total) * 100.0 } else { 0.0 };
        let reply_rate = if total > 0.0 { (c.reply_count as f64 / total) * 100.0 } else { 0.0 };

        Self {
            total_recipients: c.total_recipients,
            sent_count: c.sent_count,
            delivered_count: c.delivered_count,
            read_count: c.read_count,
            failed_count: c.failed_count,
            reply_count: c.reply_count,
            delivery_rate,
            read_rate,
            reply_rate,
        }
    }
}

// ── Request DTOs ─────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCampaignRequest {
    #[validate(length(min = 1, max = 500))]
    pub name: String,
    pub description: Option<String>,
    pub r#type: CampaignType,
    pub wa_account_id: Uuid,
    pub target_type: CampaignTargetType,
    pub target_group_id: Option<Uuid>,
    pub target_segment_id: Option<Uuid>,
    pub target_contact_ids: Option<Vec<Uuid>>,
    pub template_id: Option<Uuid>,
    pub message_body: Option<String>,
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub buttons: Option<serde_json::Value>,
    pub send_rate: Option<i32>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCampaignRequest {
    #[validate(length(min = 1, max = 500))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub message_body: Option<String>,
    pub media_url: Option<String>,
    pub template_id: Option<Uuid>,
    pub send_rate: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ScheduleCampaignRequest {
    pub scheduled_at: DateTime<Utc>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CampaignListQuery {
    pub status: Option<CampaignStatus>,
    pub r#type: Option<CampaignType>,
    pub search: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
