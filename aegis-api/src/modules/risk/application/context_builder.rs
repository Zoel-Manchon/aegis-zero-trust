//! Builds a `RiskContext` from the ports.
//!
//! This is where the application layer talks to the outside world — but only
//! through the `RiskSignalStore` and `RiskHistoryStore` *traits*, never through
//! a concrete Redis or Postgres type. That inversion is what lets the engine be
//! tested end-to-end with in-memory fakes.
//!
//! Behavioural fix vs. the original: the old builder called
//! `get_mfa_failure_count` and `get_policy_denial_count` **twice each** (once into
//! locals, then again inline when constructing the struct), doubling those Redis
//! round-trips on every authenticated request. Here each counter is read exactly
//! once.

use crate::core::errors::app_error::AppError;
use crate::modules::auth::domain::session::session::Session;
use crate::modules::risk::application::ports::{
    history_store::RiskHistoryStore, signal_store::RiskSignalStore,
};
use crate::modules::risk::domain::context::RiskContext;
use std::net::IpAddr;
use uuid::Uuid;

/// Assembles a `RiskContext` for the session being evaluated.
///
/// Generic over the two ports so production wires in the real adapters while
/// tests wire in fakes. `?Sized` + `&dyn` would also work; generics keep the
/// call sites monomorphised and avoid vtable churn on the hot auth path.
pub struct RiskContextBuilder;

impl RiskContextBuilder {
    pub async fn build<S, H>(
        signals: &S,
        history: &H,
        session: &Session,
        request_ip: IpAddr,
        request_user_agent: &str,
        jti: Uuid,
    ) -> Result<RiskContext, AppError>
    where
        S: RiskSignalStore + ?Sized,
        H: RiskHistoryStore + ?Sized,
    {
        // One round-trip for all durable aggregates.
        let hist = history
            .session_history(session.user_id, session.id, session.family_id)
            .await?;

        // Increment the request-velocity counter (this read is also a write).
        let request_count_60s = signals.record_request_velocity(session.user_id).await?;

        // Read each short-window counter exactly once (was duplicated before).
        let mfa_failure_count_10m = signals.mfa_failure_count(session.user_id).await?;
        let policy_denial_count_10m = signals.policy_denial_count(session.user_id).await?;

        Ok(RiskContext {
            user_id: session.user_id,
            session_id: session.id,
            family_id: session.family_id,
            jti,

            ip: request_ip,
            user_agent: request_user_agent.to_string(),

            original_ip: session.ip_address,
            original_user_agent: session.user_agent.clone(),

            session_created_at: session.created_at,
            last_login_at: hist.last_login_at,

            session_count_24h: hist.session_count_24h,
            unique_ip_count_24h: hist.unique_ip_count_24h,
            device_count_30d: hist.device_count_30d,
            active_family_sessions: hist.active_family_sessions,

            request_count_60s,
            policy_denial_count_10m,
            mfa_failure_count_10m,
        })
    }
}
