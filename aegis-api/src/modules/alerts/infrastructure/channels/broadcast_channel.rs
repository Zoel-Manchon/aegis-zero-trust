//! In-process broadcast channel for the admin dashboard WebSocket.
//!
//! This is the missing real-time piece: every dispatched Alert is copied into a
//! Tokio broadcast bus. The admin dashboard subscribes to that bus over
//! `/admin/security/alerts/ws`, so alerts appear without polling.

use crate::core::errors::app_error::AppError;
use crate::modules::alerts::application::channel::AlertChannel;
use crate::modules::alerts::domain::alert::Alert;
use async_trait::async_trait;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct BroadcastAlertChannel {
    sender: broadcast::Sender<Alert>,
}

impl BroadcastAlertChannel {
    pub fn new(sender: broadcast::Sender<Alert>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl AlertChannel for BroadcastAlertChannel {
    fn name(&self) -> &'static str {
        "admin-websocket"
    }

    async fn send(&self, alert: &Alert) -> Result<(), AppError> {
        // No connected receivers is not a failure: an admin console may simply
        // not be open at the time the alert is emitted.
        let _ = self.sender.send(alert.clone());
        Ok(())
    }
}
