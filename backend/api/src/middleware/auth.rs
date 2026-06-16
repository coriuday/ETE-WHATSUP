use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::user::{UserRole, UserStatus},
    utils::jwt::{verify_access_token, Claims},
};

/// Represents an authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub org_id: Option<Uuid>,
    pub org_role: Option<crate::models::organization::MemberRole>,
    pub token_version: i32,
}

/// State needed by the auth extractor
#[derive(Clone)]
pub struct AuthState {
    pub db: PgPool,
    pub jwt_secret: String,
}

/// Axum extractor that validates Bearer JWT and loads user from DB
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    axum::extract::State<crate::AppState>: FromRequestParts<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the state
        let axum::extract::State(app_state) = parts
            .extract_with_state::<axum::extract::State<crate::AppState>, S>(state)
            .await
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to extract state")))?;

        // Extract Bearer token
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        // Verify token
        let claims: Claims = verify_access_token(bearer.token(), &app_state.config.jwt_secret)
            .map_err(|_| AppError::Unauthorized)?;

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

        // Fetch user from DB (could be cached in Redis in production)
        let user = sqlx::query!(
            r#"
            SELECT id, email, role as "role: UserRole", status as "status: UserStatus",
                   token_version
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_optional(&app_state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::Unauthorized)?;

        // Check token version (invalidation)
        if user.token_version != claims.token_version {
            return Err(AppError::InvalidToken);
        }

        // Check user status
        match user.status {
            UserStatus::Suspended => return Err(AppError::AccountSuspended),
            UserStatus::PendingVerification => return Err(AppError::AccountNotVerified),
            _ => {}
        }

        // Get primary org (first org membership) and role
        let org_member = sqlx::query!(
            r#"SELECT organization_id, role as "role: crate::models::organization::MemberRole" FROM org_members WHERE user_id = $1 LIMIT 1"#,
            user_id
        )
        .fetch_optional(&app_state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(AuthUser {
            id: user.id,
            email: user.email,
            role: user.role,
            org_id: org_member.as_ref().map(|m| m.organization_id),
            org_role: org_member.map(|m| m.role),
            token_version: user.token_version,
        })
    }
}

/// Optional auth extractor — returns None if no token present
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    axum::extract::State<crate::AppState>: FromRequestParts<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let headers: &HeaderMap = &parts.headers;
        if !headers.contains_key("authorization") {
            return Ok(OptionalAuthUser(None));
        }
        let user = AuthUser::from_request_parts(parts, state).await?;
        Ok(OptionalAuthUser(Some(user)))
    }
}
