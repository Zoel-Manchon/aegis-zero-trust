use crate::{
    app_state::AppState,
    core::middleware::rate_limit::rate_limit_middleware,
    modules::{
        auth::interface::middleware::auth_middleware::auth_middleware,
        passkeys::interface::handlers::passkey_handler::{
            begin_passkey_login_handler, begin_passkey_registration_handler,
            delete_passkey_handler, finish_passkey_login_handler,
            finish_passkey_registration_handler, list_passkeys_handler,
        },
    },
};
use axum::{middleware::from_fn_with_state, routing::{delete, get, post}, Router};

pub fn passkey_routes(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/passkeys", get(list_passkeys_handler))
        .route("/passkeys", delete(delete_passkey_handler))
        .route("/passkeys/register/begin", post(begin_passkey_registration_handler))
        .route("/passkeys/register/finish", post(finish_passkey_registration_handler))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .route("/passkeys/login/begin", post(begin_passkey_login_handler))
        .route("/passkeys/login/finish", post(finish_passkey_login_handler))
        .merge(protected)
        // Hardware-key endpoints are authentication endpoints and must be
        // throttled. Add credential/user scoped throttling in passkey_service
        // when WebAuthn verification is wired.
        .route_layer(from_fn_with_state(state, rate_limit_middleware))
}
