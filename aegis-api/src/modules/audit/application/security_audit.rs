use crate::modules::alerts::application::dispatcher::AlertDispatcher;
use crate::modules::audit::{
    application::{audit_service, security_alerts},
    domain::security_event::{SecurityEventType, SecuritySeverity},
};
use crate::modules::risk::application::ports::signal_store::RiskSignalStore;
use crate::modules::risk::infrastructure::redis_signal_store::RedisRiskSignalStore;
use std::net::IpAddr;
use uuid::Uuid;

pub async fn mfa_setup_started(
    pool: &sqlx::PgPool,
    user_id: i64,
    ip: IpAddr,
    user_agent: String,
    session_id: Uuid,
    jti: Uuid,
) {
    audit_service::record_session_event(
        pool,
        Some(user_id),
        SecurityEventType::MfaRequired,
        SecuritySeverity::Info,
        Some(ip),
        Some(user_agent),
        Some(session_id),
        Some(jti),
        None,
        serde_json::json!({
            "action": "mfa_setup_started"
        }),
    )
    .await;
}

pub async fn mfa_success(
    pool: &sqlx::PgPool,
    user_id: i64,
    ip: Option<IpAddr>,
    user_agent: Option<String>,
    session_id: Option<Uuid>,
    jti: Option<Uuid>,
    action: &str,
) {
    audit_service::record_session_event(
        pool,
        Some(user_id),
        SecurityEventType::MfaSuccess,
        SecuritySeverity::Info,
        ip,
        user_agent,
        session_id,
        jti,
        None,
        serde_json::json!({
            "action": action
        }),
    )
    .await;
}

pub async fn mfa_failure(
    pool: &sqlx::PgPool,
    redis: &crate::core::cache::redis::RedisClient,
    user_id: Option<i64>,
    ip: Option<IpAddr>,
    user_agent: Option<String>,
    session_id: Option<Uuid>,
    jti: Option<Uuid>,
    action: &str,
    reason: &str,
    severity: SecuritySeverity,
) {
    if let Some(user_id) = user_id {
        let _ = RedisRiskSignalStore::new(redis.clone()).record_mfa_failure(user_id).await;
    }

    audit_service::record_session_event(
        pool,
        user_id,
        SecurityEventType::MfaFailure,
        severity,
        ip,
        user_agent,
        session_id,
        jti,
        None,
        serde_json::json!({
            "action": action,
            "reason": reason
        }),
    )
    .await;
}

/// Token purpose violation: a token presented for an action outside its scope
/// (e.g. an MFA-purpose token used at a protected route).
///
/// Now takes the AlertDispatcher and fires a high-severity alert through it.
/// Callers must pass `&state.alerts`.
pub async fn token_purpose_violation(
    pool: &sqlx::PgPool,
    alerts: &AlertDispatcher,
    ip: Option<IpAddr>,
    user_agent: Option<String>,
    action: &str,
    reason: &str,
) {
    audit_service::record_simple(
        pool,
        None,
        SecurityEventType::TokenPurposeViolation,
        SecuritySeverity::High,
        ip,
        user_agent,
        serde_json::json!({
            "action": action,
            "reason": reason
        }),
    )
    .await;

    // Fire an alert through the dispatcher.
    security_alerts::token_purpose_violation(alerts, ip, action, reason).await;
}

pub async fn login_success(
    pool: &sqlx::PgPool,
    user_id: Option<i64>,
    ip: Option<IpAddr>,
    user_agent: Option<String>,
    jti: Option<Uuid>,
    mfa: bool,
    geoip: Option<serde_json::Value>,
) {
    let mut metadata = serde_json::json!({ "mfa": mfa });
    if let Some(g) = geoip {
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert("geoip".to_string(), g);
        }
    }
    audit_service::record_session_event(
        pool,
        user_id,
        SecurityEventType::LoginSuccess,
        SecuritySeverity::Info,
        ip,
        user_agent,
        None,
        jti,
        None,
        metadata,
    )
    .await;
}

pub async fn register_success(pool: &sqlx::PgPool, email: &str) {
    audit_service::record_simple(
        pool,
        None,
        SecurityEventType::RegisterSuccess,
        SecuritySeverity::Info,
        None,
        None,
        serde_json::json!({
            "email": email
        }),
    )
    .await;
}

pub async fn register_failure(pool: &sqlx::PgPool, email: &str, reason: &str) {
    audit_service::record_simple(
        pool,
        None,
        SecurityEventType::RegisterFailure,
        SecuritySeverity::Low,
        None,
        None,
        serde_json::json!({
            "email": email,
            "reason": reason
        }),
    )
    .await;
}

pub async fn login_failure(
    pool: &sqlx::PgPool,
    email: &str,
    ip: IpAddr,
    user_agent: String,
    reason: &str,
) {
    audit_service::record_simple(
        pool,
        None,
        SecurityEventType::LoginFailure,
        SecuritySeverity::Medium,
        Some(ip),
        Some(user_agent),
        serde_json::json!({
            "email": email,
            "reason": reason
        }),
    )
    .await;
}

pub async fn mfa_required(pool: &sqlx::PgPool, email: &str, ip: IpAddr, user_agent: String) {
    audit_service::record_simple(
        pool,
        None,
        SecurityEventType::MfaRequired,
        SecuritySeverity::Medium,
        Some(ip),
        Some(user_agent),
        serde_json::json!({
            "email": email
        }),
    )
    .await;
}

pub async fn brute_force_lockout(pool: &sqlx::PgPool, email: &str, ip: IpAddr, user_agent: String) {
    audit_service::record_simple(
        pool,
        None,
        SecurityEventType::BruteForceLockout,
        SecuritySeverity::High,
        Some(ip),
        Some(user_agent),
        serde_json::json!({
            "email": email,
            "reason": "login_rate_limited"
        }),
    )
    .await;
}

pub async fn session_revoked(
    pool: &sqlx::PgPool,
    user_id: i64,
    ip: IpAddr,
    user_agent: String,
    session_id: Uuid,
    jti: Uuid,
    action: &str,
    severity: SecuritySeverity,
) {
    audit_service::record_session_event(
        pool,
        Some(user_id),
        SecurityEventType::SessionRevoked,
        severity,
        Some(ip),
        Some(user_agent),
        Some(session_id),
        Some(jti),
        None,
        serde_json::json!({
            "action": action
        }),
    )
    .await;
}

pub async fn policy_denied(
    pool: &sqlx::PgPool,
    redis: &crate::core::cache::redis::RedisClient,
    user_id: i64,
    ip: IpAddr,
    user_agent: String,
    session_id: Uuid,
    jti: Uuid,
    path: &str,
    reason: &str,
    risk_score: u8,
) {
    let _ = RedisRiskSignalStore::new(redis.clone()).record_policy_denial(user_id).await;

    audit_service::record_session_event(
        pool,
        Some(user_id),
        SecurityEventType::PolicyDenied,
        SecuritySeverity::High,
        Some(ip),
        Some(user_agent),
        Some(session_id),
        Some(jti),
        None,
        serde_json::json!({
            "path": path,
            "reason": reason,
            "risk_score": risk_score
        }),
    )
    .await;
}