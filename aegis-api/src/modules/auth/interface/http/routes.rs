use crate::{
    app_state::AppState,
    core::middleware::rate_limit::rate_limit_middleware,
    modules::auth::interface::{
        http::handlers::{
            auth_handler::{
                handler_login_user, handler_logout, handler_logout_all, handler_reg_user,
            },
            me_handler::handler_me,
            password_reset_handler::{handler_forgot_password, handler_reset_password},
            refresh_handler::handler_refresh,
            verify_email_handler::{
                handler_confirm_email_verification, handler_request_email_verification,
            },
        },
        middleware::auth_middleware::auth_middleware,
    },
};

use axum::{middleware::from_fn_with_state, routing::{get, post}, Router};

pub fn auth_routes(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/me", get(handler_me))
        .route("/logout", post(handler_logout))
        .route("/logout-all", post(handler_logout_all))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/register", post(handler_reg_user))
        .route("/login", post(handler_login_user))
        .route("/refresh", post(handler_refresh))
        .route("/password/forgot", post(handler_forgot_password))
        .route("/password/reset", post(handler_reset_password))
        // Email verification endpoints — anti-enumeration on request,
        // single-use hashed token on confirm.
        .route("/verify-email/request", post(handler_request_email_verification))
        .route("/verify-email/confirm", post(handler_confirm_email_verification))
        .merge(protected)
        .route_layer(from_fn_with_state(state, rate_limit_middleware))
}