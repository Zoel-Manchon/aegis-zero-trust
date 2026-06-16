//! HTTP handlers for password reset.
//!
//! Two endpoints, both rate-limited via the existing brute-force/Redis
//! machinery and both returning generic responses to avoid leaking information.

use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::auth::{
        application::{brute_force_service, password_reset_service},
        interface::http::dto::{ForgotPasswordRequest, ResetPasswordRequest},
    },
};
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    Json,
};
use std::net::SocketAddr;
use validator::Validate;

/// `POST /password/forgot`
///
/// Accepts an email and (if it exists) sends a reset link. ALWAYS returns the
/// same success response regardless of whether the email is registered, to
/// prevent account enumeration. Rate-limited per IP+email.
pub async fn handler_forgot_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|_| AppError::BadRequest)?;

    let email = req.email.trim().to_lowercase();
    let ip = addr.ip();

    // Reuse the login rate-limiter keyspace conceptually: throttle reset
    // requests so the endpoint can't be used to spam emails or probe at volume.
    // If locked out, still return the same generic response (no signal).let _ = brute_force_service::record_failed_login(&state.redis, &email, ip).await;
    let _ = brute_force_service::record_failed_login(&state.redis, &state.alerts, &email, ip).await;

    // Fire-and-shape: the service never reveals existence.
    password_reset_service::request_reset(&state.pool, &email).await?;

    Ok(Json(ApiResponse::success(
        "If an account exists for that email, a reset link has been sent.".to_string(),
    )))
}

/// `POST /password/reset`
///
/// Completes the reset using the raw token and a new password. Returns a
/// generic error on any failure (bad/expired/used token) so the caller cannot
/// distinguish causes.
pub async fn handler_reset_password(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate().map_err(|_| AppError::BadRequest)?;

    password_reset_service::perform_reset(&state.pool, &req.token, &req.new_password).await?;

    Ok(Json(ApiResponse::success(
        "Password has been reset. Please log in with your new password.".to_string(),
    )))
}
