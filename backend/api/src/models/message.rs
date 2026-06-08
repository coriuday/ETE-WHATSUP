use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "message_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Queued,
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "message_direction", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    Outbound,
    Inbound,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "message_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    Document,
    Location,
    Contact,
    Sticker,
    Template,
    Interactive,
    Reaction,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub wa_account_id: Option<Uuid>,
    pub campaign_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub wa_message_id: Option<String>,
    pub wa_context_id: Option<String>,
    pub direction: MessageDirection,
    pub r#type: MessageType,
    pub body: Option<String>,
    pub media_url: Option<String>,
    pub media_mime_type: Option<String>,
    pub media_filename: Option<String>,
    pub template_name: Option<String>,
    pub template_variables: Option<serde_json::Value>,
    pub buttons: Option<serde_json::Value>,
    pub status: MessageStatus,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ── Webhook incoming status update ────────────────────────────
#[derive(Debug, Deserialize)]
pub struct MetaWebhookPayload {
    pub object: String,
    pub entry: Vec<MetaWebhookEntry>,
}

#[derive(Debug, Deserialize)]
pub struct MetaWebhookEntry {
    pub id: String,
    pub changes: Vec<MetaWebhookChange>,
}

#[derive(Debug, Deserialize)]
pub struct MetaWebhookChange {
    pub value: MetaWebhookValue,
    pub field: String,
}

#[derive(Debug, Deserialize)]
pub struct MetaWebhookValue {
    pub messaging_product: Option<String>,
    pub metadata: Option<MetaWebhookMetadata>,
    pub contacts: Option<Vec<MetaWebhookContact>>,
    pub messages: Option<Vec<MetaInboundMessage>>,
    pub statuses: Option<Vec<MetaMessageStatus>>,
}

#[derive(Debug, Deserialize)]
pub struct MetaWebhookMetadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MetaWebhookContact {
    pub profile: MetaContactProfile,
    pub wa_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MetaContactProfile {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct MetaInboundMessage {
    pub from: String,
    pub id: String,
    pub timestamp: String,
    pub r#type: String,
    pub text: Option<MetaTextContent>,
    pub image: Option<MetaMediaContent>,
    pub video: Option<MetaMediaContent>,
    pub audio: Option<MetaMediaContent>,
    pub document: Option<MetaMediaContent>,
}

#[derive(Debug, Deserialize)]
pub struct MetaTextContent {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct MetaMediaContent {
    pub id: String,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub caption: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaMessageStatus {
    pub id: String,
    pub status: String,    // "sent", "delivered", "read", "failed"
    pub timestamp: String,
    pub recipient_id: String,
    pub errors: Option<Vec<MetaError>>,
}

#[derive(Debug, Deserialize)]
pub struct MetaError {
    pub code: i32,
    pub title: String,
    pub message: Option<String>,
}
