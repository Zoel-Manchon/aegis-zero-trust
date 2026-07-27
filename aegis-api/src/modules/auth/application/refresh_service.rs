use crate::{
    core::errors::app_error::AppError,
    modules::{
        alerts::application::dispatcher::AlertDispatcher,
        audit::{
            application::{audit_service, security_alerts},
            domain::security_event::{SecurityEventType, SecuritySeverity},
        },
        auth::{
            application::token_service::{generate_token_pair, hash_refresh_token},
            domain::session::session_status::SessionStatus,
            infrastructure::repositories::{
                session_repository as session_repo, user_repository::UserRepository,
            },
        },
    },
};

use chrono::Utc;
use jsonwebtoken::EncodingKey;
use subtle::ConstantTimeEq;

// ---------------------------------------------------------------------------
// REFRESH TOKEN ROTATION
// ---------------------------------------------------------------------------

pub async fn refresh_token(
    pool: &sqlx::PgPool,
    alerts: &AlertDispatcher,
    encoding_key: &EncodingKey,
    old_refresh_token: &str,
    jti: uuid::Uuid,
    refresh_secret: &str,
    request_ip: Option<std::net::IpAddr>,
    request_ua: Option<&str>,
) -> Result<(String, String, uuid::Uuid), AppError> {
    let session = session_repo::find_by_jti_raw(pool, jti)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let user_agent = request_ua.map(|ua| ua.to_string());

    if session.status == SessionStatus::Rotated {
        revoke_family_after_replay(
            pool,
            alerts,
            session.user_id,
            session.family_id,
            session.id,
            jti,
            request_ip,
            user_agent,
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    if session.status != SessionStatus::Active {
        audit_service::record_session_event(
            pool,
            Some(session.user_id),
            SecurityEventType::SessionRevoked,
            SecuritySeverity::Medium,
            request_ip,
            user_agent,
            Some(session.id),
            Some(jti),
            Some(session.family_id),
            serde_json::json!({
                "action": "refresh_rejected",
                "reason": "session_not_active",
                "status": format!("{:?}", session.status)
            }),
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    if session.expires_at <= Utc::now() {
        audit_service::record_session_event(
            pool,
            Some(session.user_id),
            SecurityEventType::SessionRevoked,
            SecuritySeverity::Medium,
            request_ip,
            user_agent,
            Some(session.id),
            Some(jti),
            Some(session.family_id),
            serde_json::json!({
                "action": "refresh_rejected",
                "reason": "session_expired"
            }),
        )
        .await;

        return Err(AppError::Unauthorized);
    }

    let incoming_hash = hash_refresh_token(old_refresh_token, refresh_secret)
        .map_err(|_| AppError::Unauthorized)?;

    let token_matches = bool::from(
        incoming_hash
            .as_bytes()
            .ct_eq(session.refresh_token_hash.as_bytes()),
    );

    if !token_matches {
        let _ = session_repo::revoke_session(pool, jti).await;

        audit_service::record_session_event(
            pool,
            Some(session.user_id),
            SecurityEventType::SessionRevoked,
            SecuritySeverity::High,
            request_ip,
            user_agent,
            Some(session.id),
            Some(jti),
            Some(session.family_id),
            serde_json::json!({
                "action": "refresh_rejected",
                "reason": "refresh_hash_mismatch"
            }),
        )
        .await;

        tracing::warn!(
            user_id = session.user_id,
            session_jti = %jti,
            "refresh token hash mismatch"
        );

        return Err(AppError::Unauthorized);
    }

    let ua_matches = matches!(request_ua, Some(ua) if ua == session.user_agent);

    if !ua_matches {
        let _ = session_repo::revoke_session(pool, jti).await;

        audit_service::record_session_event(
            pool,
            Some(session.user_id),
            SecurityEventType::SessionRevoked,
            SecuritySeverity::High,
            request_ip,
            user_agent,
            Some(session.id),
            Some(jti),
            Some(session.family_id),
            serde_json::json!({
                "action": "refresh_rejected",
                "reason": "user_agent_mismatch"
            }),
        )
        .await;

        tracing::warn!(
            user_id = session.user_id,
            session_jti = %jti,
            "refresh rejected because user-agent changed"
        );

        return Err(AppError::Unauthorized);
    }

    if let Some(ip) = request_ip {
        if ip != session.ip_address {
            audit_service::record_session_event(
                pool,
                Some(session.user_id),
                SecurityEventType::PolicyDenied,
                SecuritySeverity::Low,
                Some(ip),
                user_agent.clone(),
                Some(session.id),
                Some(jti),
                Some(session.family_id),
                serde_json::json!({
                    "action": "refresh_ip_changed",
                    "old_ip": session.ip_address.to_string(),
                    "new_ip": ip.to_string()
                }),
            )
            .await;

            tracing::warn!(
                user_id = session.user_id,
                session_jti = %jti,
                old_ip = %session.ip_address,
                new_ip = %ip,
                "refresh request IP changed"
            );
        }
    }

    let user = UserRepository::find_by_id(pool, session.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let tokens = generate_token_pair(&user, encoding_key, refresh_secret)?;

    let rotated = session_repo::rotate_session_atomic(
        pool,
        session.id,
        tokens.jti,
        tokens.refresh_token_hash,
    )
    .await?;

    let Some(new_session) = rotated else {
        revoke_family_after_replay(
            pool,
            alerts,
            session.user_id,
            session.family_id,
            session.id,
            jti,
            request_ip,
            user_agent,
        )
        .await;

        return Err(AppError::Unauthorized);
    };

    audit_service::record_session_event(
        pool,
        Some(session.user_id),
        SecurityEventType::RefreshSuccess,
        SecuritySeverity::Info,
        request_ip,
        request_ua.map(|ua| ua.to_string()),
        Some(new_session.id),
        Some(tokens.jti),
        Some(session.family_id),
        serde_json::json!({
            "action": "refresh_success",
            "rotated_from_session_id": session.id,
            "old_jti": jti,
            "new_jti": tokens.jti
        }),
    )
    .await;

    Ok((tokens.access_token, tokens.refresh_token, tokens.jti))
}

// ---------------------------------------------------------------------------
// HELPERS
// ---------------------------------------------------------------------------

async fn revoke_family_after_replay(
    pool: &sqlx::PgPool,
    alerts: &AlertDispatcher,
    user_id: i64,
    family_id: uuid::Uuid,
    session_id: uuid::Uuid,
    jti: uuid::Uuid,
    request_ip: Option<std::net::IpAddr>,
    user_agent: Option<String>,
) {
    let _ = session_repo::revoke_family(pool, family_id).await;

    audit_service::record_session_event(
        pool,
        Some(user_id),
        SecurityEventType::RefreshReplayDetected,
        SecuritySeverity::Critical,
        request_ip,
        user_agent,
        Some(session_id),
        Some(jti),
        Some(family_id),
        serde_json::json!({
            "action": "refresh_replay_detected",
            "family_revoked": true
        }),
    )
    .await;

    tracing::error!(
        user_id = user_id,
        family_id = %family_id,
        session_jti = %jti,
        "refresh token replay detected; token family revoked"
    );

    // Fire a critical alert through the dispatcher (log + email-stub + redis-stream).
    security_alerts::refresh_replay(alerts, user_id, request_ip, family_id, jti).await;
}