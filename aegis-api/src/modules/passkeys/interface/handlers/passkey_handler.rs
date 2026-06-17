use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::{
        auth::{domain::auth_result::LoginResult, interface::middleware::security_context::SecurityContext},
        passkeys::{
            application::passkey_service,
            interface::dto::passkey_dto::{
                BeginPasskeyLoginRequest, BeginPasskeyRegistrationRequest, DeletePasskeyRequest,
                FinishPasskeyLoginRequest, FinishPasskeyRegistrationRequest,
            },
        },
    },
};
use axum::{
    extract::{ConnectInfo, Extension, State},
    http::HeaderMap,
    Json,
};
use std::net::SocketAddr;
use validator::Validate;

pub async fn begin_passkey_registration_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<BeginPasskeyRegistrationRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate()?;

    let challenge = passkey_service::begin_registration(
        &state,
        ctx.user_id,
        extract_user_agent_from_headers(&headers),
        addr.ip(),
    ).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "challenge_id": challenge.challenge_id,
        "public_key": challenge.public_key,
        "friendly_name": req.friendly_name
    }))))
}

pub async fn finish_passkey_registration_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<FinishPasskeyRegistrationRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()?;

    passkey_service::finish_registration(
        &state,
        ctx.user_id,
        &req.challenge_id,
        &req.credential,
        req.friendly_name.as_deref(),
        req.transports.as_deref().unwrap_or(&[]),
    ).await?;

    Ok(Json(ApiResponse::success("Passkey registered".to_string())))
}

pub async fn begin_passkey_login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<BeginPasskeyLoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate()?;

    let challenge = passkey_service::begin_login(
        &state,
        req.email,
        extract_user_agent_from_headers(&headers),
        addr.ip(),
    ).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "challenge_id": challenge.challenge_id,
        "public_key": challenge.public_key
    }))))
}

pub async fn finish_passkey_login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<FinishPasskeyLoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    req.validate()?;

    let result = passkey_service::finish_login(
        &state,
        &req.challenge_id,
        &req.credential,
        extract_user_agent_from_headers(&headers),
        addr.ip(),
    ).await?;

    match result {
        LoginResult::Success { access_token, refresh_token, jti, .. } => Ok(Json(ApiResponse::success(serde_json::json!({
            "access_token": access_token,
            "refresh_token": refresh_token,
            "jti": jti,
            "auth_method": "passkey"
        })))),
        _ => Err(AppError::Unauthorized),
    }
}

pub async fn list_passkeys_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let passkeys = passkey_service::list_passkeys(&state, ctx.user_id).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "passkeys": passkeys
    }))))
}

pub async fn delete_passkey_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<DeletePasskeyRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    req.validate()?;

    passkey_service::revoke_passkey(&state, ctx.user_id, &req.credential_id).await?;

    Ok(Json(ApiResponse::success("Passkey revoked".to_string())))
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
