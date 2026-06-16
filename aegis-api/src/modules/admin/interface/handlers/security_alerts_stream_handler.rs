
// HARDENING NOTE:
// This SSE endpoint is behind auth_middleware via admin_routes, but long-lived
// streams need extra controls: admin-only permission, short heartbeat, max stream
// lifetime, disconnect metrics, no sensitive raw metadata, and a push-based
// broadcast channel instead of polling the DB every 5 seconds.
use crate::{app_state::AppState, modules::admin::security::application::alert_service};

use async_stream::stream;
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use std::{convert::Infallible, time::Duration};

pub async fn security_alerts_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let pool = state.pool.clone();

    let stream = stream! {
        loop {
            let payload = match alert_service::derived_security_alerts(&pool).await {
                Ok(alerts) => serde_json::json!({
                    "alerts": alerts
                }),
                Err(_) => serde_json::json!({
                    "alerts": [],
                    "error": "failed_to_load_alerts"
                }),
            };

            let data = serde_json::to_string(&payload)
                .unwrap_or_else(|_| "{\"alerts\":[]}".to_string());

            yield Ok(Event::default()
                .event("security_alerts")
                .data(data));

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    };

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    )
}
