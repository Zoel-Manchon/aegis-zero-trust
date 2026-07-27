//! Port: short-window signal counters.
//!
//! This trait is the *contract* the application layer relies on for the
//! fast-moving counters (requests in the last 60s, MFA failures in the last 10m,
//! policy denials in the last 10m). It says nothing about Redis — that's an
//! implementation detail of the adapter in `infrastructure/`.
//!
//! Because the application depends on this trait rather than on a concrete Redis
//! client, tests can supply an in-memory fake and exercise the full scoring path
//! with zero external services.

use crate::core::errors::app_error::AppError;
use async_trait::async_trait;

/// Read/write access to the per-user sliding-window counters used by the risk
/// engine. Implementations are expected to apply the appropriate TTL on first
/// increment so the windows expire on their own.
#[async_trait]
pub trait RiskSignalStore: Send + Sync {
    /// Increment and return the request count for the current 60-second window.
    async fn record_request_velocity(&self, user_id: i64) -> Result<i64, AppError>;

    /// Increment and return the MFA-failure count for the current 10-minute window.
    async fn record_mfa_failure(&self, user_id: i64) -> Result<i64, AppError>;

    /// Increment and return the policy-denial count for the current 10-minute window.
    async fn record_policy_denial(&self, user_id: i64) -> Result<i64, AppError>;

    /// Read the current MFA-failure count without incrementing (0 if none).
    async fn mfa_failure_count(&self, user_id: i64) -> Result<i64, AppError>;

    /// Read the current policy-denial count without incrementing (0 if none).
    async fn policy_denial_count(&self, user_id: i64) -> Result<i64, AppError>;
}
