use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // Authentication & Authorization
    #[error("Authentication required")]
    Unauthorized,
    #[error("Access denied: insufficient permissions")]
    Forbidden,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account is not verified")]
    AccountNotVerified,
    #[error("Account is suspended")]
    AccountSuspended,
    #[error("2FA code required")]
    TwoFactorRequired,
    #[error("Invalid 2FA code")]
    InvalidTwoFactorCode,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,

    // Resource errors
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Resource already exists: {0}")]
    Conflict(String),

    // Validation
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Bad request: {0}")]
    BadRequest(String),

    // Business logic
    #[error("Organization quota exceeded: {0}")]
    QuotaExceeded(String),
    #[error("WhatsApp account is not connected")]
    WaNotConnected,
    #[error("WhatsApp API error: {0}")]
    WhatsAppError(String),
    #[error("Campaign cannot be modified in current state")]
    InvalidCampaignState,

    // Infrastructure
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Email service error: {0}")]
    Email(String),

    // Internal
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
    #[error("Service unavailable")]
    ServiceUnavailable,

    // Rate limiting
    #[error("Too many requests")]
    RateLimited,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized
            | Self::InvalidCredentials
            | Self::TokenExpired
            | Self::InvalidToken
            | Self::TwoFactorRequired => StatusCode::UNAUTHORIZED,

            Self::Forbidden
            | Self::AccountNotVerified
            | Self::AccountSuspended
            | Self::InvalidTwoFactorCode => StatusCode::FORBIDDEN,

            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,

            Self::Validation(_) | Self::BadRequest(_) => StatusCode::BAD_REQUEST,

            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,

            Self::QuotaExceeded(_)
            | Self::WaNotConnected
            | Self::InvalidCampaignState => StatusCode::UNPROCESSABLE_ENTITY,

            Self::WhatsAppError(_) => StatusCode::BAD_GATEWAY,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,

            Self::Database(_)
            | Self::Cache(_)
            | Self::Storage(_)
            | Self::Email(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::AccountNotVerified => "ACCOUNT_NOT_VERIFIED",
            Self::AccountSuspended => "ACCOUNT_SUSPENDED",
            Self::TwoFactorRequired => "TWO_FACTOR_REQUIRED",
            Self::InvalidTwoFactorCode => "INVALID_TWO_FACTOR_CODE",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::QuotaExceeded(_) => "QUOTA_EXCEEDED",
            Self::WaNotConnected => "WA_NOT_CONNECTED",
            Self::WhatsAppError(_) => "WHATSAPP_ERROR",
            Self::InvalidCampaignState => "INVALID_CAMPAIGN_STATE",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Cache(_) => "CACHE_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Email(_) => "EMAIL_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            Self::RateLimited => "RATE_LIMITED",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code().to_string();
        let message = self.to_string();

        // Don't leak internal error details in production
        let message = match &self {
            AppError::Database(_) | AppError::Internal(_) => {
                "An internal error occurred".to_string()
            }
            _ => message,
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
