//! Port: aggregate session history.
//!
//! The slower-moving facts the engine needs (how many sessions in 24h, how many
//! distinct IPs, how many devices in 30d, etc.) come from durable storage. This
//! trait expresses that need without naming Postgres.
//!
//! Returning a single `SessionHistory` struct from one method (rather than five
//! separate getters) is deliberate: the production adapter can satisfy it in a
//! single round-trip / a small number of queries, instead of the previous code
//! which issued several independent queries AND duplicated two of them.

use crate::core::errors::app_error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Durable, aggregate facts about a user's recent session history.
#[derive(Debug, Clone)]
pub struct SessionHistory {
    pub session_count_24h: i64,
    pub unique_ip_count_24h: i64,
    pub device_count_30d: i64,
    pub active_family_sessions: i64,
    /// Most recent login *other than* the current session, if any.
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Read access to durable session history for risk scoring.
#[async_trait]
pub trait RiskHistoryStore: Send + Sync {
    /// Gather all aggregate history facts for `user_id`, excluding the current
    /// `session_id` from "last login" and counting active sessions within
    /// `family_id`.
    async fn session_history(
        &self,
        user_id: i64,
        session_id: Uuid,
        family_id: Uuid,
    ) -> Result<SessionHistory, AppError>;
}
