use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::{
        pagination::ApiResponse,
        user::{
            ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, LoginResponse,
            RegisterRequest, ResetPasswordRequest, UpdateProfileRequest, UserResponse,
        },
    },
    services::auth_service::AuthService,
    AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/verify-email", get(verify_email))
        .route("/me", get(get_me).put(update_profile))
        .route("/change-password", post(change_password))
        .route("/2fa/setup", post(setup_2fa))
        .route("/2fa/verify", post(verify_2fa))
        .route("/2fa/disable", post(disable_2fa))
        .with_state(state)
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let service = AuthService::new(&state);
    let user = service.register(req).await?;
    Ok(Json(ApiResponse::with_message(user, "Verification email sent. Please check your inbox.")))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let service = AuthService::new(&state);
    let response = service.login(req).await?;
    Ok(Json(ApiResponse::ok(response)))
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let service = AuthService::new(&state);
    let tokens = service.refresh_tokens(&req.refresh_token).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({ "tokens": tokens }))))
}

async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    let service = AuthService::new(&state);
    service.logout(auth.id, &req.refresh_token).await?;
    Ok(Json(ApiResponse::with_message((), "Logged out successfully")))
}

async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let service = AuthService::new(&state);
    service.forgot_password(&req.email).await?;
    // Always return success to prevent email enumeration
    Ok(Json(ApiResponse::with_message(
        (),
        "If that email is registered, you'll receive reset instructions.",
    )))
}

async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let service = AuthService::new(&state);
    service.reset_password(&req.token, &req.new_password).await?;
    Ok(Json(ApiResponse::with_message((), "Password reset successfully")))
}

#[derive(Deserialize)]
struct VerifyEmailQuery {
    token: String,
}

async fn verify_email(
    State(state): State<AppState>,
    Query(q): Query<VerifyEmailQuery>,
) -> AppResult<Json<ApiResponse<()>>> {
    let service = AuthService::new(&state);
    service.verify_email(&q.token).await?;
    Ok(Json(ApiResponse::with_message((), "Email verified successfully")))
}

async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let service = AuthService::new(&state);
    let user = service.get_user(auth.id).await?;
    Ok(Json(ApiResponse::ok(user)))
}

async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let service = AuthService::new(&state);
    let user = service.update_profile(auth.id, req).await?;
    Ok(Json(ApiResponse::ok(user)))
}

async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    let service = AuthService::new(&state);
    service.change_password(auth.id, &req.current_password, &req.new_password).await?;
    Ok(Json(ApiResponse::with_message((), "Password changed successfully")))
}

async fn setup_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let service = AuthService::new(&state);
    let (otpauth_url, qr_code) = service.setup_2fa(auth.id, &auth.email).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "otpauth_url": otpauth_url,
        "qr_code": qr_code
    }))))
}

#[derive(Deserialize)]
struct TotpRequest {
    code: String,
}

async fn verify_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<TotpRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    let service = AuthService::new(&state);
    service.enable_2fa(auth.id, &req.code).await?;
    Ok(Json(ApiResponse::with_message((), "2FA enabled successfully")))
}

async fn disable_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<TotpRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    let service = AuthService::new(&state);
    service.disable_2fa(auth.id, &req.code).await?;
    Ok(Json(ApiResponse::with_message((), "2FA disabled successfully")))
}
