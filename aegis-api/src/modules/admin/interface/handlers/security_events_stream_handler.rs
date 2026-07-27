// ============================================================================
// security_events_stream_handler.rs
//
// Real-time SOC push pipeline (consume side).
//
// Holds a dedicated Postgres LISTEN connection on the 'soc_events' channel and
// relays each NOTIFY (a full security_events row as JSON, emitted by the
// 0007_soc_event_notify trigger) to the connected admin dashboard as an SSE
// frame — true push, no polling.
//
// Behind admin_routes' auth_middleware (admin-only). Each connected admin gets
// its own LISTEN connection; on any listener error the stream simply ends and
// the browser reconnects.
//
// HARDENING NOTE: for many concurrent admins, replace the per-connection
// PgListener with a single process-wide listener fanned out over a
// tokio::sync::broadcast channel, and add a max-stream-lifetime cap.
// ============================================================================

use crate::app_state::AppState;

use async_stream::stream;
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use sqlx::postgres::PgListener;
use std::{convert::Infallible, time::Duration};

pub async fn security_events_stream_handler(
    State(state): State<AppState>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let pool = state.pool.clone();

    let stream = stream! {
        // Dedicated LISTEN connection. If we can't establish it, end the
        // stream cleanly; the client will retry.
        let mut listener = match PgListener::connect_with(&pool).await {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(error = ?e, "soc events stream: failed to open listener");
                return;
            }
        };

        if let Err(e) = listener.listen("soc_events").await {
            tracing::warn!(error = ?e, "soc events stream: LISTEN failed");
            return;
        }

        loop {
            match listener.recv().await {
                Ok(notification) => {
                    yield Ok(Event::default()
                        .event("soc_event")
                        .data(notification.payload().to_string()));
                }
                Err(e) => {
                    tracing::warn!(error = ?e, "soc events stream: listener error, closing");
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    )
}
