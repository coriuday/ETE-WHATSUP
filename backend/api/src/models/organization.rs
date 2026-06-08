use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "org_plan", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrgPlan {
    Free,
    Starter,
    Professional,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "org_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrgStatus {
    Active,
    Suspended,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "member_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub timezone: String,
    pub plan: OrgPlan,
    pub status: OrgStatus,
    pub max_contacts: i32,
    pub max_campaigns: i32,
    pub max_team_members: i32,
    pub monthly_msg_quota: i32,
    pub msgs_sent_this_month: i32,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOrganizationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub logo_url: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct InviteMemberRequest {
    #[validate(email)]
    pub email: String,
    pub role: MemberRole,
}
