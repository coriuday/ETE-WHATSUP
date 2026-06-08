use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "wa_account_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WaAccountStatus {
    Disconnected,
    Connecting,
    Connected,
    Banned,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "wa_account_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WaAccountType {
    CloudApi,
    BusinessApi,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct WhatsAppAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub display_name: String,
    pub phone_number: String,
    pub phone_number_id: Option<String>,
    pub waba_id: Option<String>,
    pub account_type: WaAccountType,
    pub status: WaAccountStatus,
    pub messaging_limit: Option<String>,
    pub quality_rating: Option<String>,
    pub business_name: Option<String>,
    pub business_description: Option<String>,
    pub profile_picture_url: Option<String>,
    pub total_msgs_sent: i64,
    pub last_message_at: Option<DateTime<Utc>>,
    pub connected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConnectWaAccountRequest {
    #[validate(length(min = 1, max = 255))]
    pub display_name: String,
    #[validate(length(min = 7, max = 20))]
    pub phone_number: String,
    pub phone_number_id: String,
    pub waba_id: String,
    pub access_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateWaProfileRequest {
    pub business_name: Option<String>,
    pub business_description: Option<String>,
    pub business_email: Option<String>,
    pub business_website: Option<String>,
}
