use crate::{
    app_state::AppState,
    core::{crypto::jwt, errors::app_error::AppError, response::api_response::ApiResponse},
    modules::{
        audit::{application::security_audit, domain::security_event::SecuritySeverity},
        auth::{
            application::auth_service, domain::auth_result::LoginResult,
            infrastructure::repositories::user_repository::UserRepository,
            interface::middleware::{security_context::SecurityContext, extractor_helpers::extract_client_ip},
        },
        mfa::{application::mfa_service, interface::dto::VerifyMfaRequest},
    },
};

use axum::{
    Json,
    extract::{ConnectInfo, Extension, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use validator::Validate;

#[derive(Serialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub otpauth_url: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CompleteMfaLoginRequest {
    pub mfa_token: String,

    #[validate(length(min = 6, max = 6))]
    pub code: String,
}

pub async fn setup_mfa_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ApiResponse<MfaSetupResponse>>, AppError> {
    let setup = mfa_service::setup_mfa(&state.pool, ctx.user_id, "user").await?;

    security_audit::mfa_setup_started(
        &state.pool,
        ctx.user_id,
        ctx.ip,
        ctx.user_agent,
        ctx.session_id,
        ctx.jti,
    )
    .await;

    Ok(Json(ApiResponse::success(MfaSetupResponse {
        secret: setup.secret,
        otpauth_url: setup.otpauth_url,
    })))
}

pub async fn verify_setup_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<VerifyMfaRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()?;

    let valid = mfa_service::verify_setup(&state.pool, ctx.user_id, &req.code).await?;

    if !valid {
        security_audit::mfa_failure(
            &state.pool,
            &state.redis,
            Some(ctx.user_id),
            Some(ctx.ip),
            Some(ctx.user_agent),
            Some(ctx.session_id),
            Some(ctx.jti),
            "mfa_verify_setup",
            "invalid_code",
            SecuritySeverity::Medium,
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    security_audit::mfa_success(
        &state.pool,
        ctx.user_id,
        Some(ctx.ip),
        Some(ctx.user_agent),
        Some(ctx.session_id),
        Some(ctx.jti),
        "mfa_enabled",
    )
    .await;

    Ok(Json(ApiResponse::success("MFA enabled".to_string())))
}

pub async fn verify_mfa_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<VerifyMfaRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()?;

    let valid = mfa_service::verify_code(&state.pool, ctx.user_id, &req.code).await?;

    if !valid {
        security_audit::mfa_failure(
            &state.pool,
            &state.redis,
            Some(ctx.user_id),
            Some(ctx.ip),
            Some(ctx.user_agent),
            Some(ctx.session_id),
            Some(ctx.jti),
            "mfa_verify",
            "invalid_code",
            SecuritySeverity::Medium,
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    security_audit::mfa_success(
        &state.pool,
        ctx.user_id,
        Some(ctx.ip),
        Some(ctx.user_agent),
        Some(ctx.session_id),
        Some(ctx.jti),
        "mfa_verify",
    )
    .await;

    Ok(Json(ApiResponse::success("MFA verified".to_string())))
}

pub async fn complete_mfa_login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CompleteMfaLoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate()?;

    let user_agent = extract_user_agent_from_headers(&headers);
    let ip_address = extract_client_ip(&headers, addr.ip());

    let claims = match jwt::verify_mfa_token(&req.mfa_token, &state.jwt_keys.decoding) {
        Ok(claims) => claims,
        Err(_) => {
security_audit::token_purpose_violation(
                &state.pool,
                &state.alerts,            // <-- new 2nd argument
                Some(ip_address),
                Some(user_agent),
                "mfa_complete_login",
                "invalid_mfa_token",
            )
            .await;

            return Err(AppError::Unauthorized);
        }
    };

    let valid = mfa_service::verify_code(&state.pool, claims.sub, &req.code).await?;

    if !valid {
        security_audit::mfa_failure(
            &state.pool,
            &state.redis,
            Some(claims.sub),
            Some(ip_address),
            Some(user_agent),
            None,
            None,
            "mfa_complete_login",
            "invalid_code",
            SecuritySeverity::High,
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    let user = UserRepository::find_by_id(&state.pool, claims.sub)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let result = auth_service::issue_full_login_tokens(
        &state.pool,
        &state.jwt_keys.encoding,
        &state.refresh_secret,
        &user,
        user_agent.clone(),
        ip_address,
    )
    .await?;

    match result {
        LoginResult::Success {
            access_token,
            refresh_token,
            jti,
            ..
        } => {
            security_audit::mfa_success(
                &state.pool,
                user.id,
                Some(ip_address),
                Some(user_agent.clone()),
                None,
                Some(jti),
                "mfa_complete_login",
            )
            .await;

            let geoip = crate::modules::geo::login::record_login_geo(
                &state,
                user.id,
                ip_address,
                Some(user_agent.clone()),
            )
            .await;

            security_audit::login_success(
                &state.pool,
                Some(user.id),
                Some(ip_address),
                Some(user_agent),
                Some(jti),
                true,
                geoip,
            )
            .await;

            Ok(Json(ApiResponse::success(serde_json::json!({
                "access_token": access_token,
                "refresh_token": refresh_token,
                "jti": jti
            }))))
        }

        _ => Err(AppError::Unauthorized),
    }
}

pub async fn disable_mfa_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<VerifyMfaRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()?;

    let disabled = mfa_service::disable_mfa(&state.pool, ctx.user_id, &req.code).await?;

    if !disabled {
        security_audit::mfa_failure(
            &state.pool,
            &state.redis,
            Some(ctx.user_id),
            Some(ctx.ip),
            Some(ctx.user_agent),
            Some(ctx.session_id),
            Some(ctx.jti),
            "mfa_disable",
            "invalid_code",
            SecuritySeverity::Medium,
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    security_audit::mfa_success(
        &state.pool,
        ctx.user_id,
        Some(ctx.ip),
        Some(ctx.user_agent),
        Some(ctx.session_id),
        Some(ctx.jti),
        "mfa_disabled",
    )
    .await;

    Ok(Json(ApiResponse::success("MFA disabled".to_string())))
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
