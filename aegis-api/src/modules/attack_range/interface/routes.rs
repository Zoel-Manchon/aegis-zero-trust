//! Attack-range routes. Mounted behind auth_middleware; handlers enforce admin.

use crate::{
    app_state::AppState,
    modules::{
        attack_range::interface::handlers::attack_range_handler::{
            handler_launch, handler_scenarios,
        },
        auth::interface::middleware::{admin_mfa_guard::admin_mfa_guard, auth_middleware::auth_middleware},
    },
};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

pub fn attack_range_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/attack-range/scenarios", get(handler_scenarios))
        .route("/attack-range/launch", post(handler_launch))
        .route_layer(from_fn_with_state(state.clone(), admin_mfa_guard))
        .route_layer(from_fn_with_state(state, auth_middleware))
}
