use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "subscription_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trialing,
    Active,
    PastDue,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub monthly_price_inr: rust_decimal::Decimal,
    pub yearly_price_inr: rust_decimal::Decimal,
    pub max_contacts: i32,
    pub max_campaigns: i32,
    pub max_team_members: i32,
    pub max_wa_accounts: i32,
    pub monthly_msg_quota: i32,
    pub features: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OrgSubscription {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub status: SubscriptionStatus,
    pub billing_interval: String,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
