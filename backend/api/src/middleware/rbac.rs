use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::user::UserRole,
};

/// Guard that requires a minimum role level
pub struct RequireRole(pub UserRole);

impl UserRole {
    /// Returns the numeric rank of the role (higher = more permissions)
    pub fn rank(&self) -> u8 {
        match self {
            UserRole::TeamMember => 1,
            UserRole::BusinessAdmin => 2,
            UserRole::SuperAdmin => 3,
        }
    }

    pub fn has_permission(&self, required: &UserRole) -> bool {
        self.rank() >= required.rank()
    }
}

/// Type-safe role guards as extractors

/// Requires at least TeamMember (any authenticated user)
pub struct RequireTeamMember(pub AuthUser);

/// Requires at least BusinessAdmin
pub struct RequireBusinessAdmin(pub AuthUser);

/// Requires SuperAdmin
pub struct RequireSuperAdmin(pub AuthUser);

macro_rules! impl_role_guard {
    ($guard:ty, $required_role:expr) => {
        #[async_trait]
        impl<S> FromRequestParts<S> for $guard
        where
            S: Send + Sync,
            axum::extract::State<crate::AppState>: FromRequestParts<S>,
        {
            type Rejection = AppError;

            async fn from_request_parts(
                parts: &mut Parts,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                let user = AuthUser::from_request_parts(parts, state).await?;
                if !user.role.has_permission(&$required_role) {
                    return Err(AppError::Forbidden);
                }
                Ok(Self(user))
            }
        }
    };
}

impl_role_guard!(RequireTeamMember, UserRole::TeamMember);
impl_role_guard!(RequireBusinessAdmin, UserRole::BusinessAdmin);
impl_role_guard!(RequireSuperAdmin, UserRole::SuperAdmin);
