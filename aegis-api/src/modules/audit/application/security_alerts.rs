//! Critical security event → alert dispatcher bridge.
//!
//! Each function here formats an Alert for one critical event and dispatches it
//! through the AlertDispatcher (log + email-stub + redis-stream channels).
//! Call sites import this module and call these helpers right after recording
//! the underlying audit event, so the SIEM dashboard and external channels see
//! the event in real time.
//!
//! Failures are absorbed by the dispatcher (best-effort per-channel), so a
//! down SMTP/Redis can never break the triggering auth flow.

use std::net::IpAddr;

use crate::modules::alerts::{
    application::dispatcher::AlertDispatcher,
    domain::alert::{Alert, AlertSeverity},
};

/// Fire when refresh-token replay is detected (a refresh family was reused
/// after rotation). This is a strong "token theft" signal — the legitimate
/// holder may have just been compromised.
pub async fn refresh_replay(
    alerts: &AlertDispatcher,
    user_id: i64,
    ip: Option<IpAddr>,
    family_id: uuid::Uuid,
    session_jti: uuid::Uuid,
) {
    let mut alert = Alert::new(
        "refresh_replay_detected",
        AlertSeverity::Critical,
        "Refresh token replay detected",
        format!(
            "A revoked refresh token was replayed for user {user_id}. \
             The entire token family has been revoked. If the legitimate user \
             did not just sign out, their device may be compromised."
        ),
    )
    .with_meta("user_id", user_id.to_string())
    .with_meta("family_id", family_id.to_string())
    .with_meta("session_jti", session_jti.to_string());
    if let Some(ip) = ip {
        alert = alert.with_meta("ip", ip.to_string());
    }
    alerts.dispatch(&alert).await;
}

/// Fire when brute-force protection triggers a lockout. `scope` distinguishes
/// per-IP vs per-email lockouts. Per-IP=an attacker hammering many accounts;
/// per-email=an attacker targeting one account.
pub async fn brute_force_lockout(
    alerts: &AlertDispatcher,
    scope: &str, // "ip" or "email"
    target: &str,
    lockout_seconds: u64,
) {
    let alert = Alert::new(
        "brute_force_lockout",
        AlertSeverity::High,
        format!("Brute-force lockout triggered ({scope})"),
        format!(
            "Repeated failed logins from {scope}={target} exceeded the threshold. \
             Locked out for {lockout_seconds} seconds."
        ),
    )
    .with_meta("scope", scope)
    .with_meta("target", target)
    .with_meta("lockout_seconds", lockout_seconds.to_string());

    alerts.dispatch(&alert).await;
}

/// Fire when an RBAC / policy check denies a request.
pub async fn rbac_denied(
    alerts: &AlertDispatcher,
    user_id: Option<i64>,
    ip: Option<IpAddr>,
    path: &str,
    reason: &str,
    risk_score: Option<u8>,
) {
    let mut alert = Alert::new(
        "rbac_denied",
        AlertSeverity::Medium,
        "Access denied by policy",
        format!(
            "Policy enforcement denied a request to {path}: {reason}. \
             user_id={user_id:?}."
        ),
    )
    .with_meta("path", path)
    .with_meta("reason", reason);
    if let Some(uid) = user_id {
        alert = alert.with_meta("user_id", uid.to_string());
    }
    if let Some(ip) = ip {
        alert = alert.with_meta("ip", ip.to_string());
    }
    if let Some(score) = risk_score {
        alert = alert.with_meta("risk_score", score.to_string());
    }

    alerts.dispatch(&alert).await;
}

/// Fire when a token is used for an action outside its declared purpose
/// (e.g. an MFA token presented at a protected route).
pub async fn token_purpose_violation(
    alerts: &AlertDispatcher,
    ip: Option<IpAddr>,
    action: &str,
    reason: &str,
) {
    let mut alert = Alert::new(
        "token_purpose_violation",
        AlertSeverity::High,
        "Token purpose violation",
        format!("Token purpose violation on action={action}: {reason}"),
    )
    .with_meta("action", action)
    .with_meta("reason", reason);
    if let Some(ip) = ip {
        alert = alert.with_meta("ip", ip.to_string());
    }

    alerts.dispatch(&alert).await;
}
