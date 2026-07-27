//! WebSocket alert stream for the admin dashboard.
//!
//! The old UI depended on polling/derived aggregates, so fresh alert dispatches
//! did not reliably appear. This handler subscribes to AppState's broadcast bus
//! and pushes each Alert as a small JSON frame to authenticated admins.

use crate::app_state::AppState;
use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
};
use serde_json::json;

pub async fn security_alerts_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| alert_socket(socket, state))
}

async fn alert_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.alert_bus.subscribe();

    let hello = json!({
        "type": "hello",
        "stream": "admin_alerts",
        "transport": "websocket"
    });
    if socket.send(Message::Text(hello.to_string().into())).await.is_err() {
        return;
    }

    loop {
        match rx.recv().await {
            Ok(alert) => {
                let frame = json!({
                    "type": "alert",
                    "alert": {
                        "alert_type": alert.kind,
                        "severity": alert.severity.as_str(),
                        "title": alert.title,
                        "description": alert.body,
                        "count": 1,
                        "metadata": alert.metadata,
                    }
                });
                if socket.send(Message::Text(frame.to_string().into())).await.is_err() {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                let frame = json!({ "type": "lagged", "skipped": skipped });
                if socket.send(Message::Text(frame.to_string().into())).await.is_err() {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}
