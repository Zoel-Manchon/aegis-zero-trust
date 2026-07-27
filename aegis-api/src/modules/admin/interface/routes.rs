use crate::{
    app_state::AppState,
    modules::{
        admin::interface::handlers::{
            admin_handler::admin_dashboard_handler,
            security_alerts_handler::security_alerts_handler,
            security_alerts_stream_handler::security_alerts_stream_handler,
            security_alerts_ws_handler::security_alerts_ws_handler,
            security_events_handler::security_events_handler,
            security_events_stream_handler::security_events_stream_handler,
            security_metrics_handler::security_metrics_handler,
        },
        auth::interface::middleware::{admin_mfa_guard::admin_mfa_guard, auth_middleware::auth_middleware},
    },
};

use axum::{Router, middleware::from_fn_with_state, routing::get};

pub fn admin_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/admin/dashboard", get(admin_dashboard_handler))
        .route("/admin/security/events", get(security_events_handler))
        .route("/admin/security/events/stream", get(security_events_stream_handler))
        .route("/admin/security/metrics", get(security_metrics_handler))
        .route("/admin/security/alerts", get(security_alerts_handler))
        .route(
            "/admin/security/alerts/stream",
            get(security_alerts_stream_handler),
        )
        .route("/admin/security/alerts/ws", get(security_alerts_ws_handler))
        .route_layer(from_fn_with_state(state.clone(), admin_mfa_guard))
        .route_layer(from_fn_with_state(state, auth_middleware))
}
