use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "conversation_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ConversationStatus {
    Open,
    Pending,
    Resolved,
    Snoozed,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub wa_account_id: Option<Uuid>,
    pub contact_id: Uuid,
    pub status: ConversationStatus,
    pub session_expires_at: Option<DateTime<Utc>>,
    pub is_in_session: bool,
    pub assigned_to: Option<Uuid>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub subject: Option<String>,
    pub first_message_at: Option<DateTime<Utc>>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub last_message_body: Option<String>,
    pub unread_count: i32,
    pub labels: Vec<String>,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationListQuery {
    pub status: Option<ConversationStatus>,
    pub assigned_to: Option<Uuid>,
    pub search: Option<String>,
    pub label: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub message_type: String,
    pub body: Option<String>,
    pub media_url: Option<String>,
    pub template_id: Option<Uuid>,
    pub template_variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct AssignConversationRequest {
    pub user_id: Option<Uuid>,
}
