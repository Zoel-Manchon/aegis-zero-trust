use crate::modules::admin::security::domain::security_alert::SecurityAlert;
use crate::modules::admin::security::domain::security_event_view::SecurityEventView;
use crate::modules::admin::security::domain::security_metric::SecurityMetrics;
pub async fn list_security_events(
    pool: &sqlx::PgPool,
    limit: i64,
) -> Result<Vec<SecurityEventView>, sqlx::Error> {
    sqlx::query_as!(
        SecurityEventView,
        r#"
        SELECT
            id,
            user_id,
            event_type,
            severity,
            ip_address as "ip_address: std::net::IpAddr",
            user_agent,
            session_id,
            jti,
            family_id,
            metadata,
            created_at
        FROM security_events
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn security_metrics(pool: &sqlx::PgPool) -> Result<SecurityMetrics, sqlx::Error> {
    sqlx::query_as!(
        SecurityMetrics,
        r#"
        SELECT
            COALESCE(COUNT(*), 0)::bigint
                AS "total_events!",

            COALESCE(
                COUNT(*) FILTER (
                    WHERE severity = 'CRITICAL'
                ),
                0
            )::bigint
                AS "critical_events!",

            COALESCE(
                COUNT(*) FILTER (
                    WHERE severity = 'HIGH'
                ),
                0
            )::bigint
                AS "high_events!",

            COALESCE(
                COUNT(*) FILTER (
                    WHERE event_type = 'REFRESH_REPLAY_DETECTED'
                ),
                0
            )::bigint
                AS "refresh_replays!",

            COALESCE(
                COUNT(*) FILTER (
                    WHERE event_type = 'POLICY_DENIED'
                ),
                0
            )::bigint
                AS "policy_denials!",

            COALESCE(
                COUNT(*) FILTER (
                    WHERE event_type = 'MFA_FAILURE'
                ),
                0
            )::bigint
                AS "mfa_failures!",

            COALESCE(
                COUNT(*) FILTER (
                    WHERE event_type = 'BRUTE_FORCE_LOCKOUT'
                ),
                0
            )::bigint
                AS "brute_force_lockouts!"

        FROM security_events
        "#
    )
    .fetch_one(pool)
    .await
}

pub async fn derived_security_alerts(
    pool: &sqlx::PgPool,
) -> Result<Vec<SecurityAlert>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (
                WHERE severity = 'CRITICAL'
                  AND created_at > now() - interval '24 hours'
            )::bigint AS "critical_events!",

            COUNT(*) FILTER (
                WHERE event_type = 'POLICY_DENIED'
                  AND created_at > now() - interval '10 minutes'
            )::bigint AS "policy_denials_10m!",

            COUNT(*) FILTER (
                WHERE event_type = 'MFA_FAILURE'
                  AND created_at > now() - interval '10 minutes'
            )::bigint AS "mfa_failures_10m!",

            COUNT(*) FILTER (
                WHERE event_type = 'BRUTE_FORCE_LOCKOUT'
                  AND created_at > now() - interval '10 minutes'
            )::bigint AS "brute_force_lockouts_10m!",

            COUNT(*) FILTER (
                WHERE event_type = 'REFRESH_REPLAY_DETECTED'
                  AND created_at > now() - interval '24 hours'
            )::bigint AS "refresh_replays_24h!",

            COUNT(*) FILTER (
                WHERE (metadata->'impossible_travel'->>'detected')::boolean IS TRUE
                  AND created_at > now() - interval '24 hours'
            )::bigint AS "impossible_travel_24h!"
        FROM security_events
        "#
    )
    .fetch_one(pool)
    .await?;

    let mut alerts = Vec::new();

    if rows.critical_events > 0 {
        alerts.push(SecurityAlert {
            alert_type: "CRITICAL_EVENTS_24H".to_string(),
            severity: "CRITICAL".to_string(),
            title: "Critical security events detected".to_string(),
            description: "One or more critical security events occurred in the last 24 hours."
                .to_string(),
            count: rows.critical_events,
        });
    }

    if rows.policy_denials_10m >= 5 {
        alerts.push(SecurityAlert {
            alert_type: "POLICY_DENIAL_SPIKE".to_string(),
            severity: "HIGH".to_string(),
            title: "Policy denial spike".to_string(),
            description: "Multiple policy denials occurred in the last 10 minutes.".to_string(),
            count: rows.policy_denials_10m,
        });
    }

    if rows.mfa_failures_10m >= 5 {
        alerts.push(SecurityAlert {
            alert_type: "MFA_FAILURE_SPIKE".to_string(),
            severity: "HIGH".to_string(),
            title: "MFA failure spike".to_string(),
            description: "Multiple MFA failures occurred in the last 10 minutes.".to_string(),
            count: rows.mfa_failures_10m,
        });
    }

    if rows.brute_force_lockouts_10m >= 3 {
        alerts.push(SecurityAlert {
            alert_type: "BRUTE_FORCE_LOCKOUT_SPIKE".to_string(),
            severity: "HIGH".to_string(),
            title: "Brute-force lockout spike".to_string(),
            description: "Multiple brute-force lockouts occurred in the last 10 minutes."
                .to_string(),
            count: rows.brute_force_lockouts_10m,
        });
    }

    if rows.refresh_replays_24h > 0 {
        alerts.push(SecurityAlert {
            alert_type: "REFRESH_REPLAY_DETECTED".to_string(),
            severity: "CRITICAL".to_string(),
            title: "Refresh token replay detected".to_string(),
            description: "At least one refresh token replay was detected in the last 24 hours."
                .to_string(),
            count: rows.refresh_replays_24h,
        });
    }

    if rows.impossible_travel_24h > 0 {
        alerts.push(SecurityAlert {
            alert_type: "IMPOSSIBLE_TRAVEL".to_string(),
            severity: "CRITICAL".to_string(),
            title: "Impossible travel detected".to_string(),
            description: "A user appeared from two distant GeoIP locations faster than the travel threshold allows.".to_string(),
            count: rows.impossible_travel_24h,
        });
    }

    Ok(alerts)
}
