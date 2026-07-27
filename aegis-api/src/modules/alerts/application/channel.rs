//! Port: an alert delivery channel.
//!
//! Each delivery mechanism (log, email, websocket, Slack, PagerDuty, ...) is an
//! adapter implementing this trait. The dispatcher depends only on the trait,
//! so adding a channel never touches the dispatch logic, and tests can use a
//! recording fake.

use crate::core::errors::app_error::AppError;
use crate::modules::alerts::domain::alert::Alert;
use async_trait::async_trait;

#[async_trait]
pub trait AlertChannel: Send + Sync {
    /// A short name for logging/diagnostics, e.g. "log", "email", "websocket".
    fn name(&self) -> &'static str;

    /// Deliver one alert. Channels should be best-effort and fast; the
    /// dispatcher isolates failures so one bad channel can't block the others.
    async fn send(&self, alert: &Alert) -> Result<(), AppError>;
}
