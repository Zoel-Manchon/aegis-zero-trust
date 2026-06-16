use crate::core::errors::app_error::AppError;
use axum::http::{HeaderMap, header};

pub fn extract_bearer(headers: &HeaderMap) -> Result<&str, AppError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .ok_or(AppError::Unauthorized)?;

    let value = value.to_str().map_err(|_| AppError::Unauthorized)?;

    let token = value
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    if token.is_empty() || token.chars().any(char::is_whitespace) {
        return Err(AppError::Unauthorized);
    }

    Ok(token)
}

pub fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .chars()
        .take(512)
        .collect()
}


pub fn extract_client_ip(headers: &HeaderMap, fallback: std::net::IpAddr) -> std::net::IpAddr {
    for name in ["x-forwarded-for", "x-real-ip", "cf-connecting-ip"] {
        if let Some(ip) = headers.get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .and_then(|v| v.parse().ok()) {
            return ip;
        }
    }
    fallback
}
