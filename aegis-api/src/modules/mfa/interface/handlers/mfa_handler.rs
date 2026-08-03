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
        mfa::{
            application::mfa_service::{self, SecondFactor},
            interface::dto::VerifyMfaRequest,
        },
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

    /// A 6-digit TOTP code, or a backup code. Widened from exactly 6 so the
    /// recovery path works when the authenticator is gone.
    #[validate(length(min = 6, max = 32))]
    pub code: String,
}

/// Returned exactly once, when MFA is enabled or the codes are regenerated.
/// Nothing else in the system can ever show these again — only their Argon2
/// hashes are stored.
#[derive(Serialize)]
pub struct BackupCodesResponse {
    pub backup_codes: Vec<String>,
}

#[derive(Serialize)]
pub struct BackupCodesStatusResponse {
    pub remaining: i64,
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

/// Confirm enrollment. Returns the freshly minted backup codes — the only time
/// they are ever visible, so the response type changed from a bare message.
pub async fn verify_setup_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<VerifyMfaRequest>,
) -> Result<Json<ApiResponse<BackupCodesResponse>>, AppError> {
    req.validate()?;

    let issued = mfa_service::verify_setup(&state.pool, ctx.user_id, &req.code).await?;

    let Some(backup_codes) = issued else {
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
    };

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

    Ok(Json(ApiResponse::success(BackupCodesResponse { backup_codes })))
}

/// Mint a new set of backup codes, invalidating the old ones. Requires a valid
/// second factor: otherwise a stolen session could quietly issue itself a
/// permanent way back in.
pub async fn regenerate_backup_codes_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<VerifyMfaRequest>,
) -> Result<Json<ApiResponse<BackupCodesResponse>>, AppError> {
    req.validate()?;

    if mfa_service::verify_second_factor(&state.pool, ctx.user_id, &req.code)
        .await?
        .is_none()
    {
        security_audit::mfa_failure(
            &state.pool,
            &state.redis,
            Some(ctx.user_id),
            Some(ctx.ip),
            Some(ctx.user_agent),
            Some(ctx.session_id),
            Some(ctx.jti),
            "mfa_backup_regenerate",
            "invalid_code",
            SecuritySeverity::High,
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    let backup_codes = mfa_service::regenerate_backup_codes(&state.pool, ctx.user_id).await?;

    security_audit::mfa_success(
        &state.pool,
        ctx.user_id,
        Some(ctx.ip),
        Some(ctx.user_agent),
        Some(ctx.session_id),
        Some(ctx.jti),
        "mfa_backup_codes_regenerated",
    )
    .await;

    Ok(Json(ApiResponse::success(BackupCodesResponse { backup_codes })))
}

/// How many codes are left. Read-only and cheap, so the console can warn
/// before the user runs out rather than after.
pub async fn backup_codes_status_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ApiResponse<BackupCodesStatusResponse>>, AppError> {
    let remaining = mfa_service::remaining_backup_codes(&state.pool, ctx.user_id).await?;

    Ok(Json(ApiResponse::success(BackupCodesStatusResponse { remaining })))
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

    let factor = mfa_service::verify_second_factor(&state.pool, claims.sub, &req.code).await?;

    let Some(factor) = factor else {
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
    };

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
            // A recovery-code login is recorded under its own action so it
            // stands out in the SOC feed: it is legitimate, but it is also
            // exactly what an attacker without the device would use.
            security_audit::mfa_success(
                &state.pool,
                user.id,
                Some(ip_address),
                Some(user_agent.clone()),
                None,
                Some(jti),
                match factor {
                    SecondFactor::Totp => "mfa_complete_login",
                    SecondFactor::BackupCode => "mfa_complete_login_backup_code",
                },
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
