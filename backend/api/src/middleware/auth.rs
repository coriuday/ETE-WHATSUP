use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        organization::MemberRole,
        user::{UserRole, UserStatus},
    },
    utils::jwt::{verify_access_token, Claims},
};

const AUTH_CACHE_TTL_SECS: u64 = 60;
pub const ORG_HEADER: &str = "x-organization-id";

/// Represents an authenticated user extracted from JWT + optional org context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub org_id: Option<Uuid>,
    pub org_role: Option<MemberRole>,
    pub token_version: i32,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    axum::extract::State<crate::AppState>: FromRequestParts<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::State(app_state) = parts
            .extract_with_state::<axum::extract::State<crate::AppState>, S>(state)
            .await
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to extract state")))?;

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let claims: Claims = verify_access_token(bearer.token(), &app_state.config.jwt_secret)
            .map_err(|_| AppError::Unauthorized)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

        let header_org_id = parts
            .headers
            .get(ORG_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s.trim()).ok());

        let cache_key = format!(
            "auth:user:{}:org:{}",
            user_id,
            header_org_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "none".into())
        );

        if let Ok(mut conn) = app_state.redis.get().await {
            if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
                if let Ok(user) = serde_json::from_str::<AuthUser>(&cached) {
                    if user.token_version == claims.token_version {
                        return Ok(user);
                    }
                }
            }
        }

        let row = sqlx::query_as::<_, (Uuid, String, UserRole, UserStatus, i32, bool)>(
            r#"
            SELECT id, email, role, status, token_version, email_verified
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

        let (id, email, role, status, token_version, email_verified) = row;

        if token_version != claims.token_version {
            return Err(AppError::InvalidToken);
        }

        match status {
            UserStatus::Suspended => return Err(AppError::AccountSuspended),
            UserStatus::PendingVerification => return Err(AppError::AccountNotVerified),
            _ => {}
        }

        if !email_verified {
            return Err(AppError::AccountNotVerified);
        }

        let (org_id, org_role) =
            resolve_org_membership(&app_state, id, &role, header_org_id).await?;

        let auth_user = AuthUser {
            id,
            email,
            role,
            org_id,
            org_role,
            token_version,
        };

        if let Ok(mut conn) = app_state.redis.get().await {
            if let Ok(json) = serde_json::to_string(&auth_user) {
                let _: Result<(), _> = conn.set_ex(&cache_key, json, AUTH_CACHE_TTL_SECS).await;
            }
        }

        Ok(auth_user)
    }
}

async fn resolve_org_membership(
    app_state: &crate::AppState,
    user_id: Uuid,
    role: &UserRole,
    header_org_id: Option<Uuid>,
) -> Result<(Option<Uuid>, Option<MemberRole>), AppError> {
    if let Some(org_id) = header_org_id {
        if *role == UserRole::SuperAdmin {
            return Ok((Some(org_id), Some(MemberRole::Owner)));
        }

        let membership = sqlx::query_as::<_, (MemberRole,)>(
            r#"
            SELECT role FROM org_members
            WHERE user_id = $1 AND organization_id = $2
            "#,
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(AppError::Database)?;

        return match membership {
            Some((m,)) => Ok((Some(org_id), Some(m))),
            None => Err(AppError::Forbidden),
        };
    }

    let memberships = sqlx::query_as::<_, (Uuid, MemberRole)>(
        r#"
        SELECT organization_id, role
        FROM org_members
        WHERE user_id = $1
        LIMIT 2
        "#,
    )
    .bind(user_id)
    .fetch_all(&app_state.db)
    .await
    .map_err(AppError::Database)?;

    match memberships.len() {
        1 => {
            tracing::debug!(
                user_id = %user_id,
                "X-Organization-Id missing; using sole org membership"
            );
            Ok((Some(memberships[0].0), Some(memberships[0].1.clone())))
        }
        _ => Ok((None, None)),
    }
}

pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    axum::extract::State<crate::AppState>: FromRequestParts<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if !parts.headers.contains_key(axum::http::header::AUTHORIZATION) {
            return Ok(OptionalAuthUser(None));
        }
        let user = AuthUser::from_request_parts(parts, state).await?;
        Ok(OptionalAuthUser(Some(user)))
    }
}
