use crate::app_state::AppState;
use crate::core::errors::app_error::AppError;
use crate::modules::auth::application::refresh_service;
use crate::modules::auth::interface::middleware::extractor_helpers::extract_client_ip;

use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
    pub jti: String,
}

pub async fn handler_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let jti = uuid::Uuid::parse_str(&req.jti).map_err(|_| AppError::Unauthorized)?;

    let user_agent: String = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown-device")
        .to_string();

    let ip_address = extract_client_ip(&headers, addr.ip());

    let (access, refresh, new_jti) = refresh_service::refresh_token(
        &state.pool,
        &state.alerts,              // <-- AlertDispatcher passed through
        &state.jwt_keys.encoding,
        &req.refresh_token,
        jti,
        &state.refresh_secret,
        Some(ip_address),
        Some(user_agent.as_str()),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "access_token": access,
        "refresh_token": refresh,
        "jti": new_jti
    })))
}