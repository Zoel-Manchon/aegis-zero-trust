//! Redis-backed stream channel.
//!
//! Publishes the latest alert as JSON under a well-known Redis key with a short
//! TTL. The admin SSE stream (`security_alerts_stream_handler`) can be extended
//! to read this key so freshly-dispatched alerts surface to connected admin
//! dashboards in near-real-time, complementing the periodic DB-derived view.
//!
//! Named "websocket" to match the original scaffold filename; the transport is
//! actually Server-Sent Events + Redis, but the role is the same: push live
//! alerts to connected clients.

use crate::core::cache::redis::RedisClient;
use crate::core::errors::app_error::AppError;
use crate::modules::alerts::application::channel::AlertChannel;
use crate::modules::alerts::domain::alert::Alert;
use async_trait::async_trait;

/// Key the SSE handler can read to pick up the most recent pushed alert.
pub const LATEST_ALERT_KEY: &str = "alerts:latest";
/// How long a pushed alert stays readable (seconds).
const ALERT_TTL_SECS: usize = 60;

pub struct RedisStreamChannel {
    redis: RedisClient,
}

impl RedisStreamChannel {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}

#[async_trait]
impl AlertChannel for RedisStreamChannel {
    fn name(&self) -> &'static str {
        "websocket"
    }

    async fn send(&self, alert: &Alert) -> Result<(), AppError> {
        let payload = serde_json::to_string(alert).map_err(|_| AppError::InternalError)?;
        self.redis
            .set_ex(LATEST_ALERT_KEY, &payload, ALERT_TTL_SECS)
            .await
            .map_err(|_| AppError::InternalError)?;
        Ok(())
    }
}
