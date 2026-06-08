use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "contact_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ContactStatus {
    Active,
    Blocked,
    Unsubscribed,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "contact_source", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ContactSource {
    Manual,
    CsvImport,
    ExcelImport,
    Api,
    Form,
    WhatsappInbound,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Contact {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub phone_number: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub wa_status: ContactStatus,
    pub wa_opted_in: bool,
    pub wa_opted_in_at: Option<DateTime<Utc>>,
    pub opted_out_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub source: ContactSource,
    pub custom_fields: serde_json::Value,
    pub total_msgs_received: i32,
    pub total_msgs_sent: i32,
    pub last_contacted_at: Option<DateTime<Utc>>,
    pub last_replied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Request DTOs ─────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateContactRequest {
    #[validate(length(min = 7, max = 20))]
    pub phone_number: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
    pub group_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
    pub wa_status: Option<ContactStatus>,
}

#[derive(Debug, Deserialize)]
pub struct ContactListQuery {
    pub search: Option<String>,
    pub status: Option<ContactStatus>,
    pub tags: Option<String>,          // comma-separated
    pub group_id: Option<Uuid>,
    pub segment_id: Option<Uuid>,
    pub source: Option<ContactSource>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BulkActionRequest {
    pub contact_ids: Vec<Uuid>,
    pub action: BulkContactAction,
    pub group_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkContactAction {
    AddToGroup,
    RemoveFromGroup,
    AddTags,
    RemoveTags,
    Delete,
    Unsubscribe,
}

// ── Contact Group ─────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ContactGroup {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub contact_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGroupRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}
