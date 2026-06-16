//! Global HTTP hardening layer.
//!
//! Defense-in-depth middleware applied to every route via main.rs:
//!   * Strict security response headers (HSTS, X-Frame-Options, nosniff,
//!     Referrer-Policy, restrictive CSP, Permissions-Policy).
//!   * CORS policy (strict in production, dev-friendly locally).
//!   * Request body size cap (DoS mitigation).
//!   * Global request timeout (sheds slow/hung requests).
//!
//! Env tunables:
//!   APP_ENV                 = "production" | "development"  (default development)
//!   CORS_ALLOWED_ORIGINS    = comma-separated origins (required in production)
//!   MAX_BODY_BYTES          = body cap, default 1 MiB
//!   REQUEST_TIMEOUT_SECS    = global timeout, default 15
//!
//! Fails safe: APP_ENV=production with no origins => deny all cross-origin.

use axum::http::{header, HeaderValue, Method};
use axum::Router;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

fn is_production() -> bool {
    std::env::var("APP_ENV").map(|v| v == "production").unwrap_or(false)
}

fn max_body_bytes() -> usize {
    std::env::var("MAX_BODY_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1024 * 1024)
}

fn request_timeout() -> Duration {
    let secs = std::env::var("REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);
    Duration::from_secs(secs)
}

fn cors_layer() -> CorsLayer {
    if is_production() {
        let origins = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
        let parsed: Vec<HeaderValue> = origins
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        let mut cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::DELETE])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
            .allow_credentials(true);

        if parsed.is_empty() {
            tracing::warn!(
                "APP_ENV=production but CORS_ALLOWED_ORIGINS is empty — denying all cross-origin"
            );
        } else {
            cors = cors.allow_origin(parsed);
        }
        cors
    } else {
        CorsLayer::new()
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_origin(Any)
    }
}

fn static_header(
    name: header::HeaderName,
    value: &'static str,
) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(name, HeaderValue::from_static(value))
}

/// Wrap a router with the full hardening stack.
pub fn apply(router: Router) -> Router {
    let prod = is_production();

    let mut r = router
        .layer(TimeoutLayer::new(request_timeout()))
        .layer(RequestBodyLimitLayer::new(max_body_bytes()))
        .layer(cors_layer())
        .layer(static_header(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(static_header(header::X_FRAME_OPTIONS, "DENY"))
        .layer(static_header(header::REFERRER_POLICY, "no-referrer"))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'none'; frame-ancestors 'none'; base-uri 'none'",
            ),
        ));

    if prod {
        r = r.layer(static_header(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains; preload",
        ));
    }

    r
}
