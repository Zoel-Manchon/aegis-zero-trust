//! The `RiskContext` value object.
//!
//! This is the complete, already-gathered set of facts the scoring logic needs
//! to evaluate a request. It is deliberately a *plain data* struct: the
//! application layer's `RiskContextBuilder` is responsible for populating it
//! from the ports (Redis + Postgres), and the domain scoring functions consume
//! it without any awareness of where the numbers came from.
//!
//! Moving this into `domain/` (it previously lived in `risk/model.rs`) makes the
//! dependency direction clean: signals and the engine import from `domain`, never
//! from a loose top-level module.

use chrono::{DateTime, Utc};
use std::net::IpAddr;
use uuid::Uuid;

/// All inputs required to score a single authenticated request.
#[derive(Debug, Clone)]
pub struct RiskContext {
    // --- identity of the request/session being scored ---
    pub user_id: i64,
    pub session_id: Uuid,
    pub family_id: Uuid,
    pub jti: Uuid,

    // --- "now" values: where this request is coming from ---
    pub ip: IpAddr,
    pub user_agent: String,

    // --- "baseline" values: what the session was originally bound to ---
    pub original_ip: IpAddr,
    pub original_user_agent: String,

    // --- temporal facts ---
    pub session_created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,

    // --- aggregate history (from Postgres) ---
    pub session_count_24h: i64,
    pub unique_ip_count_24h: i64,
    pub device_count_30d: i64,
    pub active_family_sessions: i64,

    // --- short-window counters (from Redis) ---
    pub request_count_60s: i64,
    pub policy_denial_count_10m: i64,
    pub mfa_failure_count_10m: i64,
}