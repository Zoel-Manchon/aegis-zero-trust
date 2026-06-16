//! Log channel: always-available delivery that writes the alert to the tracing
//! log. This is the safe default channel — it has no external dependencies and
//! never fails, so every deployment has at least one working sink.

use crate::core::errors::app_error::AppError;
use crate::modules::alerts::application::channel::AlertChannel;
use crate::modules::alerts::domain::alert::Alert;
use async_trait::async_trait;

pub struct LogChannel;

#[async_trait]
impl AlertChannel for LogChannel {
    fn name(&self) -> &'static str {
        "log"
    }

    async fn send(&self, alert: &Alert) -> Result<(), AppError> {
        tracing::info!(
            target: "alerts",
            kind = %alert.kind,
            severity = alert.severity.as_str(),
            title = %alert.title,
            recipient = alert.recipient.as_deref().unwrap_or("-"),
            "{}",
            alert.body
        );
        Ok(())
    }
}
