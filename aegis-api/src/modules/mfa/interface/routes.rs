use crate::{
    app_state::AppState,
    core::middleware::rate_limit::rate_limit_middleware,
    modules::{
        auth::interface::middleware::auth_middleware::auth_middleware,
        mfa::interface::handlers::mfa_handler::{
            complete_mfa_login_handler, disable_mfa_handler, setup_mfa_handler, verify_mfa_handler,
            verify_setup_handler,
        },
    },
};

use axum::{middleware::from_fn_with_state, routing::post, Router};

pub fn mfa_routes(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/mfa/setup", post(setup_mfa_handler))
        .route("/mfa/verify-setup", post(verify_setup_handler))
        .route("/mfa/verify", post(verify_mfa_handler))
        .route("/mfa/disable", post(disable_mfa_handler))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/mfa/complete-login", post(complete_mfa_login_handler))
        .merge(protected)
        // Zero Trust note: MFA endpoints are high-value brute-force targets.
        // Keep a coarse per-IP limiter here, and add a stricter user/token-scoped
        // MFA limiter in mfa_service before production.
        .route_layer(from_fn_with_state(state, rate_limit_middleware))
}
