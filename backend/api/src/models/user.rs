use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    SuperAdmin,
    BusinessAdmin,
    TeamMember,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified: bool,
    pub two_factor_enabled: bool,
    #[serde(skip_serializing)]
    pub two_factor_secret: Option<String>,
    pub token_version: i32,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Public user response (no sensitive fields)
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified: bool,
    pub two_factor_enabled: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            full_name: format!("{} {}", u.first_name, u.last_name),
            id: u.id,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            avatar_url: u.avatar_url,
            role: u.role,
            status: u.status,
            email_verified: u.email_verified,
            two_factor_enabled: u.two_factor_enabled,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
        }
    }
}

// ── Request DTOs ──────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    pub org_name: Option<String>,
    pub invite_token: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

// ── Response DTOs ────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub tokens: AuthTokens,
    pub requires_2fa: bool,
}
