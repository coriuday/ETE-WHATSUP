pub mod dispatch;
pub mod factory;
pub mod lifecycle;
pub mod meta;
pub mod mock;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Mock,
    Meta,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mock => "mock",
            Self::Meta => "meta",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "meta" | "cloud_api" | "whatsapp" => Self::Meta,
            _ => Self::Mock,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub kind: ProviderKind,
    pub phone_number_id: String,
    pub access_token: String,
    pub display_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendResult {
    pub provider_message_id: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct OutboundText {
    pub to: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct OutboundTemplate {
    pub to: String,
    pub template_name: String,
    pub language: String,
    pub components: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct OutboundMedia {
    pub to: String,
    pub media_url: String,
    pub caption: Option<String>,
    pub mime_type: Option<String>,
}

#[async_trait]
pub trait MessagingProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;

    async fn send_message(&self, account: &ProviderAccount, msg: OutboundText) -> AppResult<SendResult>;

    async fn send_template(
        &self,
        account: &ProviderAccount,
        msg: OutboundTemplate,
    ) -> AppResult<SendResult>;

    async fn send_media(&self, account: &ProviderAccount, msg: OutboundMedia) -> AppResult<SendResult>;

    async fn get_message_status(&self, provider_message_id: &str) -> AppResult<String>;

    async fn process_webhook(&self, payload: serde_json::Value) -> AppResult<()>;

    async fn health_check(&self, account: &ProviderAccount) -> AppResult<String>;
}
