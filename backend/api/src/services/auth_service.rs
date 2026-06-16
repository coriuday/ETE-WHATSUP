use anyhow::Result;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::user::{
        AuthTokens, LoginRequest, LoginResponse, RegisterRequest, UpdateProfileRequest,
        UserResponse,
    },
    utils::{
        encryption::{hash_password, verify_password},
        jwt::{
            generate_access_token, generate_refresh_token, generate_secure_token,
            verify_refresh_token,
        },
    },
    AppState,
};

pub struct AuthService<'a> {
    state: &'a AppState,
}

impl<'a> AuthService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub async fn register(&self, req: RegisterRequest) -> AppResult<UserResponse> {
        // Check email uniqueness
        let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", req.email)
            .fetch_optional(&self.state.db)
            .await
            .map_err(AppError::Database)?;

        if existing.is_some() {
            return Err(AppError::Conflict("Email already registered".into()));
        }

        let password_hash = hash_password(&req.password)
            .map_err(|e| AppError::Internal(e))?;

        let verify_token = generate_secure_token(32);
        let verify_expires = Utc::now() + Duration::hours(24);

        let user = sqlx::query!(
            r#"
            INSERT INTO users (
                email, password_hash, first_name, last_name,
                role, status, email_verify_token, email_verify_expires_at
            ) VALUES ($1, $2, $3, $4, 'team_member', 'pending_verification', $5, $6)
            RETURNING id, email, first_name, last_name, role::text, status::text,
                      email_verified, two_factor_enabled, created_at
            "#,
            req.email,
            password_hash,
            req.first_name,
            req.last_name,
            verify_token,
            verify_expires
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        // Create org if org_name provided
        if let Some(org_name) = req.org_name {
            if !org_name.is_empty() {
                let slug = slug::slugify(&org_name);
                let org = sqlx::query!(
                    r#"
                    INSERT INTO organizations (name, slug, owner_id)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (slug) DO UPDATE SET slug = $2 || '-' || substr(md5(random()::text), 1, 4)
                    RETURNING id
                    "#,
                    org_name,
                    slug,
                    user.id
                )
                .fetch_one(&self.state.db)
                .await
                .map_err(AppError::Database)?;

                sqlx::query!(
                    "INSERT INTO org_members (organization_id, user_id, role) VALUES ($1, $2, 'owner')",
                    org.id,
                    user.id
                )
                .execute(&self.state.db)
                .await
                .map_err(AppError::Database)?;
            }
        }

        // Send verification email (fire-and-forget)
        let email_service = crate::services::email_service::EmailService::new(self.state);
        let _ = email_service
            .send_verification_email(&req.email, &req.first_name, &verify_token)
            .await;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            full_name: format!("{} {}", user.first_name, user.last_name),
            avatar_url: None,
            role: crate::models::user::UserRole::TeamMember,
            status: crate::models::user::UserStatus::PendingVerification,
            email_verified: user.email_verified,
            two_factor_enabled: user.two_factor_enabled,
            last_login_at: None,
            created_at: user.created_at,
        })
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<LoginResponse> {
        let user = sqlx::query_as!(
            crate::models::user::User,
            r#"
            SELECT id, email, password_hash, first_name, last_name, avatar_url,
                   role as "role: _", status as "status: _", email_verified,
                   two_factor_enabled, two_factor_secret, token_version,
                   last_login_at, created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            req.email
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::InvalidCredentials)?;

        // Verify password
        let valid = verify_password(&req.password, &user.password_hash)
            .map_err(|e| AppError::Internal(e))?;
        if !valid {
            return Err(AppError::InvalidCredentials);
        }

        // Check status
        match user.status {
            crate::models::user::UserStatus::Suspended => return Err(AppError::AccountSuspended),
            crate::models::user::UserStatus::PendingVerification => return Err(AppError::AccountNotVerified),
            _ => {}
        }

        // Check 2FA
        if user.two_factor_enabled {
            if let Some(totp_code) = &req.totp_code {
                // Verify TOTP
                if let Some(secret) = &user.two_factor_secret {
                    let decrypted = crate::utils::encryption::decrypt(secret, &self.state.config.encryption_key)
                        .map_err(|e| AppError::Internal(e))?;
                    let totp = totp_rs::TOTP::new(
                        totp_rs::Algorithm::SHA1, 6, 1, 30,
                        decrypted.as_bytes().to_vec(), None, user.email.clone(),
                    ).map_err(|_| AppError::Internal(anyhow::anyhow!("TOTP error")))?;

                    if !totp.check_current(totp_code).unwrap_or(false) {
                        return Err(AppError::InvalidTwoFactorCode);
                    }
                }
            } else {
                // No code provided — request it
                return Ok(LoginResponse {
                    user: user.clone().into(),
                    tokens: AuthTokens {
                        access_token: String::new(),
                        refresh_token: String::new(),
                        token_type: "Bearer".into(),
                        expires_in: 0,
                    },
                    requires_2fa: true,
                });
            }
        }

        // Generate tokens
        let role_str = match user.role {
            crate::models::user::UserRole::SuperAdmin => "super_admin",
            crate::models::user::UserRole::BusinessAdmin => "business_admin",
            crate::models::user::UserRole::TeamMember => "team_member",
        };

        let access_token = generate_access_token(
            user.id,
            &user.email,
            role_str,
            user.token_version,
            &self.state.config.jwt_secret,
            self.state.config.jwt_access_expires_secs,
        )
        .map_err(|e| AppError::Internal(e))?;

        let refresh_token = generate_refresh_token(
            user.id,
            &user.email,
            role_str,
            user.token_version,
            &self.state.config.jwt_refresh_secret,
            self.state.config.jwt_refresh_expires_secs,
        )
        .map_err(|e| AppError::Internal(e))?;

        // Store session (hashed refresh token)
        let token_hash = crate::utils::encryption::sha256_hex(&refresh_token);
        sqlx::query!(
            "INSERT INTO user_sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
            user.id,
            token_hash,
            Utc::now() + Duration::seconds(self.state.config.jwt_refresh_expires_secs as i64)
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        // Update last login
        sqlx::query!(
            "UPDATE users SET last_login_at = NOW() WHERE id = $1",
            user.id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(LoginResponse {
            user: user.into(),
            tokens: AuthTokens {
                access_token,
                refresh_token,
                token_type: "Bearer".into(),
                expires_in: self.state.config.jwt_access_expires_secs,
            },
            requires_2fa: false,
        })
    }

    pub async fn refresh_tokens(&self, refresh_token: &str) -> AppResult<AuthTokens> {
        let claims = verify_refresh_token(refresh_token, &self.state.config.jwt_refresh_secret)
            .map_err(|_| AppError::InvalidToken)?;

        let token_hash = crate::utils::encryption::sha256_hex(refresh_token);

        // Verify session exists and not revoked
        let session = sqlx::query!(
            "SELECT id FROM user_sessions WHERE token_hash = $1 AND revoked = FALSE AND expires_at > NOW()",
            token_hash
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        if session.is_none() {
            return Err(AppError::InvalidToken);
        }

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

        let user = sqlx::query!(
            "SELECT token_version, email, role::text FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::Unauthorized)?;

        if user.token_version != claims.token_version {
            return Err(AppError::InvalidToken);
        }

        // Rotate: revoke old refresh token
        sqlx::query!(
            "UPDATE user_sessions SET revoked = TRUE WHERE token_hash = $1",
            token_hash
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let role = user.role.as_deref().unwrap_or("team_member");

        let new_access = generate_access_token(
            user_id, &user.email, role, user.token_version,
            &self.state.config.jwt_secret, self.state.config.jwt_access_expires_secs,
        ).map_err(|e| AppError::Internal(e))?;

        let new_refresh = generate_refresh_token(
            user_id, &user.email, role, user.token_version,
            &self.state.config.jwt_refresh_secret, self.state.config.jwt_refresh_expires_secs,
        ).map_err(|e| AppError::Internal(e))?;

        let new_token_hash = crate::utils::encryption::sha256_hex(&new_refresh);

        sqlx::query!(
            "INSERT INTO user_sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
            user_id,
            new_token_hash,
            Utc::now() + Duration::seconds(self.state.config.jwt_refresh_expires_secs as i64)
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(AuthTokens {
            access_token: new_access,
            refresh_token: new_refresh,
            token_type: "Bearer".into(),
            expires_in: self.state.config.jwt_access_expires_secs,
        })
    }

    pub async fn logout(&self, user_id: Uuid, refresh_token: &str) -> AppResult<()> {
        let token_hash = crate::utils::encryption::sha256_hex(refresh_token);
        sqlx::query!(
            "UPDATE user_sessions SET revoked = TRUE WHERE user_id = $1 AND token_hash = $2",
            user_id,
            token_hash
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;
        Ok(())
    }

    pub async fn forgot_password(&self, email: &str) -> AppResult<()> {
        let user = sqlx::query!(
            "SELECT id, first_name FROM users WHERE email = $1 AND deleted_at IS NULL",
            email
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        if let Some(user) = user {
            let token = generate_secure_token(32);
            let expires = Utc::now() + Duration::hours(2);

            sqlx::query!(
                "UPDATE users SET password_reset_token = $1, password_reset_expires_at = $2 WHERE id = $3",
                token, expires, user.id
            )
            .execute(&self.state.db)
            .await
            .map_err(AppError::Database)?;

            let email_service = crate::services::email_service::EmailService::new(self.state);
            let _ = email_service.send_password_reset_email(email, &user.first_name, &token).await;
        }

        Ok(())
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> AppResult<()> {
        let user = sqlx::query!(
            "SELECT id FROM users WHERE password_reset_token = $1 AND password_reset_expires_at > NOW()",
            token
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::InvalidToken)?;

        let hash = hash_password(new_password).map_err(|e| AppError::Internal(e))?;

        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1, password_reset_token = NULL, password_reset_expires_at = NULL,
                token_version = token_version + 1
            WHERE id = $2
            "#,
            hash, user.id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn verify_email(&self, token: &str) -> AppResult<()> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET email_verified = TRUE, status = 'active',
                email_verify_token = NULL, email_verify_expires_at = NULL
            WHERE email_verify_token = $1 AND email_verify_expires_at > NOW()
            RETURNING id
            "#,
            token
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        if result.is_none() {
            return Err(AppError::InvalidToken);
        }

        Ok(())
    }

    pub async fn get_user(&self, user_id: Uuid) -> AppResult<UserResponse> {
        let user = sqlx::query_as!(
            crate::models::user::User,
            r#"
            SELECT id, email, password_hash, first_name, last_name, avatar_url,
                   role as "role: _", status as "status: _", email_verified,
                   two_factor_enabled, two_factor_secret, token_version,
                   last_login_at, created_at, updated_at, deleted_at
            FROM users WHERE id = $1 AND deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User".into()))?;

        Ok(user.into())
    }

    pub async fn update_profile(&self, user_id: Uuid, req: UpdateProfileRequest) -> AppResult<UserResponse> {
        sqlx::query!(
            r#"
            UPDATE users
            SET first_name = COALESCE($1, first_name),
                last_name = COALESCE($2, last_name),
                avatar_url = COALESCE($3, avatar_url)
            WHERE id = $4
            "#,
            req.first_name, req.last_name, req.avatar_url, user_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        self.get_user(user_id).await
    }

    pub async fn change_password(&self, user_id: Uuid, current: &str, new: &str) -> AppResult<()> {
        let user = sqlx::query!("SELECT password_hash FROM users WHERE id = $1", user_id)
            .fetch_one(&self.state.db)
            .await
            .map_err(AppError::Database)?;

        let valid = verify_password(current, &user.password_hash)
            .map_err(|e| AppError::Internal(e))?;
        if !valid {
            return Err(AppError::InvalidCredentials);
        }

        let hash = hash_password(new).map_err(|e| AppError::Internal(e))?;
        sqlx::query!(
            "UPDATE users SET password_hash = $1, token_version = token_version + 1 WHERE id = $2",
            hash, user_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn setup_2fa(&self, user_id: Uuid, email: &str) -> AppResult<(String, String)> {
        let secret = totp_rs::Secret::generate_secret();
        let secret_str = secret.to_string();

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1, 6, 1, 30,
            secret.to_bytes().unwrap(),
            Some("WhatsUp Platform".to_string()),
            email.to_string(),
        ).map_err(|_| AppError::Internal(anyhow::anyhow!("TOTP setup error")))?;

        let otpauth_url = totp.get_url();

        // Temporarily store encrypted secret (not enabled until verified)
        let encrypted = crate::utils::encryption::encrypt(&secret_str, &self.state.config.encryption_key)
            .map_err(|e| AppError::Internal(e))?;

        sqlx::query!(
            "UPDATE users SET two_factor_secret = $1 WHERE id = $2",
            encrypted, user_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        // Generate QR code as data URL
        let qr_code = totp.get_qr_base64()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("QR code error")))?;

        Ok((otpauth_url, format!("data:image/png;base64,{}", qr_code)))
    }

    pub async fn enable_2fa(&self, user_id: Uuid, code: &str) -> AppResult<()> {
        let user = sqlx::query!(
            "SELECT two_factor_secret, email FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        let secret = user.two_factor_secret.ok_or(AppError::BadRequest("2FA not set up".into()))?;
        let decrypted = crate::utils::encryption::decrypt(&secret, &self.state.config.encryption_key)
            .map_err(|e| AppError::Internal(e))?;

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1, 6, 1, 30,
            decrypted.as_bytes().to_vec(), None, user.email,
        ).map_err(|_| AppError::Internal(anyhow::anyhow!("TOTP error")))?;

        if !totp.check_current(code).unwrap_or(false) {
            return Err(AppError::InvalidTwoFactorCode);
        }

        sqlx::query!(
            "UPDATE users SET two_factor_enabled = TRUE WHERE id = $1",
            user_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn disable_2fa(&self, user_id: Uuid, code: &str) -> AppResult<()> {
        // Verify code before disabling
        self.enable_2fa(user_id, code).await?; // reuse verification

        sqlx::query!(
            "UPDATE users SET two_factor_enabled = FALSE, two_factor_secret = NULL WHERE id = $1",
            user_id
        )
        .execute(&self.state.db)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

}
