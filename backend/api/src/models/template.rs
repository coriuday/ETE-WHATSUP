use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "template_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Marketing,
    Utility,
    Authentication,
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateCategory::Marketing => write!(f, "marketing"),
            TemplateCategory::Utility => write!(f, "utility"),
            TemplateCategory::Authentication => write!(f, "authentication"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "template_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TemplateStatus {
    Draft,
    PendingApproval,
    Approved,
    Rejected,
    Disabled,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Template {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub wa_account_id: Option<Uuid>,
    pub meta_template_id: Option<String>,
    pub name: String,
    pub display_name: String,
    pub category: TemplateCategory,
    pub language: String,
    pub status: TemplateStatus,
    pub header: Option<serde_json::Value>,
    pub body_text: String,
    pub body_example_vars: Option<Vec<String>>,
    pub footer_text: Option<String>,
    pub buttons: Option<serde_json::Value>,
    pub variable_count: i32,
    pub variable_definitions: serde_json::Value,
    pub usage_count: i32,
    pub rejection_reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTemplateRequest {
    #[validate(length(min = 1, max = 512))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub display_name: String,
    pub category: TemplateCategory,
    pub language: String,
    pub wa_account_id: Uuid,
    pub header: Option<serde_json::Value>,
    #[validate(length(min = 1))]
    pub body_text: String,
    pub body_example_vars: Option<Vec<String>>,
    pub footer_text: Option<String>,
    pub buttons: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateListQuery {
    pub status: Option<TemplateStatus>,
    pub category: Option<TemplateCategory>,
    pub search: Option<String>,
    pub wa_account_id: Option<Uuid>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
