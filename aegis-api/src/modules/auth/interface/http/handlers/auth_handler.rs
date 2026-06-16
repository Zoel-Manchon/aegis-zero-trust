use crate::modules::auth::application::brute_force_service;
use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::{
        audit::{application::security_audit, domain::security_event::SecuritySeverity},
        auth::{
            application::auth_service,
            domain::auth_result::{LoginResult, RegisterResult},
            interface::{
                http::dto::{LoginRequest, RegisterRequest},
                middleware::{security_context::SecurityContext, extractor_helpers::extract_client_ip},
            },
        },
    },
};

use axum::{
    Json,
    extract::{ConnectInfo, Extension, State},
    http::{HeaderMap, StatusCode},
};
use std::net::SocketAddr;
use validator::Validate;

pub async fn handler_reg_user(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()?;

    let email = req.email.trim().to_lowercase();

    let result = auth_service::register_user(&state.pool, email.clone(), req.password).await?;

    match &result {
        RegisterResult::Success => {
            security_audit::register_success(&state.pool, &email).await;
        }
        RegisterResult::WeakPassword => {
            security_audit::register_failure(&state.pool, &email, "weak_password").await;
        }
        RegisterResult::EmailAlreadyExists => {
            security_audit::register_failure(&state.pool, &email, "email_exists").await;
        }
        RegisterResult::InvalidCredentials => {
            security_audit::register_failure(&state.pool, &email, "invalid_credentials").await;
        }
    }

    let response = match result {
        RegisterResult::Success => ApiResponse::success("User created".to_string()),
        RegisterResult::WeakPassword => ApiResponse::error("Weak password", "AUTH_WEAK_PASSWORD"),
        RegisterResult::EmailAlreadyExists => {
            ApiResponse::error("Email already exists", "AUTH_EMAIL_EXISTS")
        }
        RegisterResult::InvalidCredentials => {
            ApiResponse::error("Invalid credentials", "AUTH_INVALID_CREDENTIALS")
        }
    };

    Ok(Json(response))
}

pub async fn handler_login_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate()?;

    let normalized_email = req.email.trim().to_lowercase();
    let user_agent = extract_user_agent_from_headers(&headers);
    let ip_address = extract_client_ip(&headers, addr.ip());

    let allowed =
        brute_force_service::check_login_allowed(&state.redis, &normalized_email, ip_address).await;

    if let Err(AppError::RateLimited) = allowed {
        security_audit::brute_force_lockout(
            &state.pool,
            &normalized_email,
            ip_address,
            user_agent.clone(),
        )
        .await;

        return Err(AppError::RateLimited);
    }

    allowed?;

    let result = auth_service::login_user(
        &state.pool,
        &state.jwt_keys.encoding,
        &state.refresh_secret,
        normalized_email.clone(),
        req.password,
        user_agent.clone(),
        ip_address,
    )
    .await?;

    match &result {
        LoginResult::Success { user_id, jti, .. } => {
            brute_force_service::clear_failed_login(&state.redis, &normalized_email, ip_address)
                .await?;

            let geoip = crate::modules::geo::login::record_login_geo(
                &state,
                *user_id,
                ip_address,
                Some(user_agent.clone()),
            )
            .await;

            security_audit::login_success(
                &state.pool,
                Some(*user_id),
                Some(ip_address),
                Some(user_agent.clone()),
                Some(*jti),
                false,
                geoip,
            )
            .await;
        }

        LoginResult::MfaRequired { user_id, .. } => {
            brute_force_service::clear_failed_login(&state.redis, &normalized_email, ip_address)
                .await?;

            security_audit::mfa_required(
                &state.pool,
                &normalized_email,
                ip_address,
                user_agent.clone(),
            )
            .await;
        }

        LoginResult::InvalidCredentials => {
         brute_force_service::record_failed_login(&state.redis, &state.alerts, &normalized_email, ip_address)
                .await?;

            security_audit::login_failure(
                &state.pool,
                &normalized_email,
                ip_address,
                user_agent.clone(),
                "invalid_credentials",
            )
            .await;
        }
    }

    let response = match result {
        LoginResult::Success {
            access_token,
            refresh_token,
            jti,
            ..
        } => ApiResponse::success(serde_json::json!({
            "access_token": access_token,
            "refresh_token": refresh_token,
            "jti": jti
        })),

        LoginResult::MfaRequired { mfa_token, .. } => ApiResponse::success(serde_json::json!({
            "mfa_required": true,
            "mfa_token": mfa_token
        })),

        LoginResult::InvalidCredentials => {
            return Err(AppError::Unauthorized);
        }
    };

    Ok(Json(response))
}

pub async fn handler_logout(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, AppError> {
    auth_service::logout(&state.pool, ctx.jti).await?;

    security_audit::session_revoked(
        &state.pool,
        ctx.user_id,
        ctx.ip,
        ctx.user_agent,
        ctx.session_id,
        ctx.jti,
        "logout",
        SecuritySeverity::Info,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn handler_logout_all(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, AppError> {
    auth_service::logout_all(&state.pool, ctx.user_id).await?;

    security_audit::session_revoked(
        &state.pool,
        ctx.user_id,
        ctx.ip,
        ctx.user_agent,
        ctx.session_id,
        ctx.jti,
        "logout_all",
        SecuritySeverity::Medium,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

fn extract_user_agent_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown-device")
        .chars()
        .take(512)
        .collect()
}
