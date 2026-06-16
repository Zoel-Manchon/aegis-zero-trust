//! Tamper-evident insert + verification for the audit chain.

use crate::modules::audit::domain::hash_chain::{compute_event_hash, GENESIS_PREV_HASH};
use crate::modules::audit::domain::security_event::NewSecurityEvent;
use serde::Serialize;
use sqlx::{PgPool, Row};

const CHAIN_LOCK_KEY: i64 = 0x5EC0_0DED;

pub async fn insert_chained_event(
    pool: &PgPool, event: NewSecurityEvent,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(CHAIN_LOCK_KEY).execute(&mut *tx).await?;

    let head = sqlx::query(
        "SELECT seq, event_hash FROM security_events WHERE seq IS NOT NULL ORDER BY seq DESC LIMIT 1",
    ).fetch_optional(&mut *tx).await?;

    let (prev_seq, prev_hash) = match head {
        Some(row) => (row.try_get::<i64, _>("seq")?, row.try_get::<String, _>("event_hash")?),
        None => (0, GENESIS_PREV_HASH.to_string()),
    };
    let seq = prev_seq + 1;

    let created_at = chrono::Utc::now();
    let created_at_str = created_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    let ip_str = event.ip_address.map(|ip| ip.to_string());
    let sid_str = event.session_id.map(|u| u.to_string());
    let jti_str = event.jti.map(|u| u.to_string());
    let fid_str = event.family_id.map(|u| u.to_string());
    let meta_str = crate::modules::audit::domain::hash_chain::canonical_json(&event.metadata);

    let event_hash = compute_event_hash(
        seq, &prev_hash, event.user_id, event.event_type.as_str(),
        event.severity.as_str(), ip_str.as_deref(), sid_str.as_deref(),
        jti_str.as_deref(), fid_str.as_deref(), &meta_str, &created_at_str,
    );

    sqlx::query(
        r#"
        INSERT INTO security_events (
            user_id, event_type, severity, ip_address, user_agent,
            session_id, jti, family_id, metadata, created_at,
            seq, prev_hash, event_hash
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
        "#,
    )
    .bind(event.user_id).bind(event.event_type.as_str()).bind(event.severity.as_str())
    .bind(event.ip_address).bind(event.user_agent).bind(event.session_id)
    .bind(event.jti).bind(event.family_id).bind(&event.metadata).bind(created_at)
    .bind(seq).bind(&prev_hash).bind(&event_hash)
    .execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ChainVerification {
    pub verified: bool,
    pub events_checked: i64,
    pub broken_at_seq: Option<i64>,
    pub reason: Option<String>,
}

pub async fn verify_chain(pool: &PgPool) -> Result<ChainVerification, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT seq, prev_hash, event_hash, user_id, event_type, severity,
               ip_address, session_id, jti, family_id, metadata, created_at
        FROM security_events WHERE seq IS NOT NULL ORDER BY seq ASC
        "#,
    ).fetch_all(pool).await?;

    let mut expected_prev = GENESIS_PREV_HASH.to_string();
    let mut checked: i64 = 0;

    for row in rows {
        let seq: i64 = row.try_get("seq")?;
        let stored_prev: String = row.try_get("prev_hash")?;
        let stored_hash: String = row.try_get("event_hash")?;
        let user_id: Option<i64> = row.try_get("user_id")?;
        let event_type: String = row.try_get("event_type")?;
        let severity: String = row.try_get("severity")?;
        let ip: Option<std::net::IpAddr> = row.try_get("ip_address")?;
        let session_id: Option<uuid::Uuid> = row.try_get("session_id")?;
        let jti: Option<uuid::Uuid> = row.try_get("jti")?;
        let family_id: Option<uuid::Uuid> = row.try_get("family_id")?;
        let metadata: serde_json::Value = row.try_get("metadata")?;
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;

        if stored_prev != expected_prev {
            return Ok(ChainVerification {
                verified: false, events_checked: checked,
                broken_at_seq: Some(seq),
                reason: Some(format!("prev_hash mismatch at seq {seq}: chain link broken")),
            });
        }

        let ip_str = ip.map(|addr| addr.to_string());
        let recomputed = compute_event_hash(
            seq, &stored_prev, user_id, &event_type, &severity,
            ip_str.as_deref(), session_id.map(|u| u.to_string()).as_deref(),
            jti.map(|u| u.to_string()).as_deref(),
            family_id.map(|u| u.to_string()).as_deref(),
            &crate::modules::audit::domain::hash_chain::canonical_json(&metadata), &created_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        );

        if recomputed != stored_hash {
            return Ok(ChainVerification {
                verified: false, events_checked: checked,
                broken_at_seq: Some(seq),
                reason: Some(format!("event_hash mismatch at seq {seq}: row contents modified")),
            });
        }

        expected_prev = stored_hash;
        checked += 1;
    }

    Ok(ChainVerification { verified: true, events_checked: checked, broken_at_seq: None, reason: None })
}
