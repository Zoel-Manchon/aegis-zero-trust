//! Edge rate limiting: per-IP request throttling.
//!
//! Complements the existing login brute-force service (which is email+IP
//! specific) with a GENERAL per-IP limiter applied as middleware to sensitive
//! routes. Uses the same Redis sliding-window primitive (`incr_with_ttl`) as
//! brute-force, so behaviour is consistent and there's one rate-limit story.
//!
//! Applied selectively in the route builders to endpoints that are cheap to
//! abuse: /register, /login, /password/forgot, /password/reset, /refresh.
//!
//! Env tunables:
//!   RATE_LIMIT_WINDOW_SECS  (default 60)
//!   RATE_LIMIT_MAX_REQUESTS (default 30 per window per IP)
//!
//! Returns 429 (AppError::RateLimited) when exceeded.

use crate::app_state::AppState;
use crate::core::errors::app_error::AppError;
use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

fn window_secs() -> usize {
    std::env::var("RATE_LIMIT_WINDOW_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60)
}

fn max_requests() -> i64 {
    std::env::var("RATE_LIMIT_MAX_REQUESTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
}

/// Axum middleware: throttle by client IP. Attach with
/// `from_fn_with_state(state, rate_limit_middleware)` on the routes you want
/// protected.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let ip = addr.ip();

    // Per-IP fixed-window counter. Keyed by the path's first segment so e.g.
    // /password/forgot and /login share a sensible bucket per IP but distinct
    // from unrelated traffic.
    let path = request.uri().path();
    let bucket = path.split('/').nth(1).unwrap_or("root");
    let key = format!("rl:ip:{ip}:{bucket}");

    let count = state
        .redis
        .incr_with_ttl(&key, window_secs())
        .await
        .map_err(|_| AppError::InternalError)?;

    if count > max_requests() {
        tracing::warn!(%ip, bucket, count, "rate limit exceeded");
        return Err(AppError::RateLimited);
    }

    Ok(next.run(request).await)
}
