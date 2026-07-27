//! Redis adapter for `RiskSignalStore`.
//!
//! This is an *adapter*: it implements the application's port using the concrete
//! `RedisClient`. The application never sees this type directly — it only sees
//! the `RiskSignalStore` trait. Swapping Redis for something else means writing a
//! new adapter here and changing one wiring line in `AppState`; no application or
//! domain code changes.

use crate::core::cache::redis::RedisClient;
use crate::core::errors::app_error::AppError;
use crate::modules::risk::application::ports::signal_store::RiskSignalStore;
use async_trait::async_trait;

/// TTLs for the sliding windows, in seconds.
const REQUEST_WINDOW_SECS: usize = 60;
const TEN_MINUTES_SECS: usize = 600;

/// Wraps the shared Redis connection manager. Cheap to clone (it just clones the
/// underlying `ConnectionManager` handle).
#[derive(Clone)]
pub struct RedisRiskSignalStore {
    redis: RedisClient,
}

impl RedisRiskSignalStore {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    fn request_key(user_id: i64) -> String {
        format!("risk:req:user:{user_id}:60s")
    }

    fn mfa_fail_key(user_id: i64) -> String {
        format!("risk:mfa_fail:user:{user_id}:10m")
    }

    fn policy_denied_key(user_id: i64) -> String {
        format!("risk:policy_denied:user:{user_id}:10m")
    }
}

#[async_trait]
impl RiskSignalStore for RedisRiskSignalStore {
    async fn record_request_velocity(&self, user_id: i64) -> Result<i64, AppError> {
        self.redis
            .incr_with_ttl(&Self::request_key(user_id), REQUEST_WINDOW_SECS)
            .await
            .map_err(|_| AppError::InternalError)
    }

    async fn record_mfa_failure(&self, user_id: i64) -> Result<i64, AppError> {
        self.redis
            .incr_with_ttl(&Self::mfa_fail_key(user_id), TEN_MINUTES_SECS)
            .await
            .map_err(|_| AppError::InternalError)
    }

    async fn record_policy_denial(&self, user_id: i64) -> Result<i64, AppError> {
        self.redis
            .incr_with_ttl(&Self::policy_denied_key(user_id), TEN_MINUTES_SECS)
            .await
            .map_err(|_| AppError::InternalError)
    }

    async fn mfa_failure_count(&self, user_id: i64) -> Result<i64, AppError> {
        Ok(self
            .redis
            .get_i64(&Self::mfa_fail_key(user_id))
            .await
            .map_err(|_| AppError::InternalError)?
            .unwrap_or(0))
    }

    async fn policy_denial_count(&self, user_id: i64) -> Result<i64, AppError> {
        Ok(self
            .redis
            .get_i64(&Self::policy_denied_key(user_id))
            .await
            .map_err(|_| AppError::InternalError)?
            .unwrap_or(0))
    }
}
