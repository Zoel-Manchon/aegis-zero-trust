//! Postgres adapter for `RiskHistoryStore`.
//!
//! Implements the history port using `sqlx`. The five aggregate facts that were
//! previously gathered as separate inline queries in the old `context_builder`
//! now live here, behind the trait. This keeps all SQL for risk-history in one
//! place and out of the application layer.
//!
//! Note on query style: these use the runtime-checked `query_as`/`query!` forms
//! rather than the compile-time `query_as!` macro so the crate still builds
//! without a live database at compile time. If you run `cargo sqlx prepare`
//! against the rebuilt schema, you can switch these to the macro form for
//! compile-time verification.

use crate::core::errors::app_error::AppError;
use crate::modules::risk::application::ports::history_store::{RiskHistoryStore, SessionHistory};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Wraps the shared connection pool.
#[derive(Clone)]
pub struct PgRiskHistoryStore {
    pool: PgPool,
}

impl PgRiskHistoryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RiskHistoryStore for PgRiskHistoryStore {
    async fn session_history(
        &self,
        user_id: i64,
        session_id: Uuid,
        family_id: Uuid,
    ) -> Result<SessionHistory, AppError> {
        // Sessions created in the last 24h.
        let session_count_24h: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM sessions
            WHERE user_id = $1
              AND created_at > now() - interval '24 hours'
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Distinct source IPs in the last 24h.
        let unique_ip_count_24h: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT ip_address)
            FROM sessions
            WHERE user_id = $1
              AND created_at > now() - interval '24 hours'
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Distinct devices (by user_agent) in the last 30 days.
        let device_count_30d: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT user_agent)
            FROM sessions
            WHERE user_id = $1
              AND created_at > now() - interval '30 days'
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Active sessions within this family (should be 1; >1 is a red flag).
        let active_family_sessions: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM sessions
            WHERE family_id = $1
              AND status = 'active'
            "#,
        )
        .bind(family_id)
        .fetch_one(&self.pool)
        .await?;

        // Most recent prior login, excluding the current session.
        let last_login_at: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
            r#"
            SELECT created_at
            FROM sessions
            WHERE user_id = $1
              AND id <> $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(SessionHistory {
            session_count_24h: session_count_24h.0,
            unique_ip_count_24h: unique_ip_count_24h.0,
            device_count_30d: device_count_30d.0,
            active_family_sessions: active_family_sessions.0,
            last_login_at: last_login_at.map(|row| row.0),
        })
    }
}
