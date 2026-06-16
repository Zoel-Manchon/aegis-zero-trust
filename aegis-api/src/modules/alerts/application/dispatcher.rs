//! The alert dispatcher: fan one alert out to every configured channel.
//!
//! Failures are isolated and logged, never propagated — a down SMTP server must
//! not break the request that triggered the alert. The dispatcher holds its
//! channels as trait objects, so the set is chosen once at startup (in
//! AppState) and is trivial to extend or mock.

use crate::modules::alerts::application::channel::AlertChannel;
use crate::modules::alerts::domain::alert::Alert;
use std::sync::Arc;

#[derive(Clone)]
pub struct AlertDispatcher {
    channels: Vec<Arc<dyn AlertChannel>>,
}

impl AlertDispatcher {
    pub fn new(channels: Vec<Arc<dyn AlertChannel>>) -> Self {
        Self { channels }
    }

    /// Dispatch to all channels. Best-effort: a channel error is logged and
    /// swallowed so other channels still fire and the caller is never blocked.
    pub async fn dispatch(&self, alert: &Alert) {
        for channel in &self.channels {
            if let Err(e) = channel.send(alert).await {
                tracing::warn!(
                    channel = channel.name(),
                    kind = %alert.kind,
                    error = ?e,
                    "alert channel delivery failed"
                );
            }
        }
    }
}
