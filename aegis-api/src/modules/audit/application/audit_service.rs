use crate::modules::audit::{
    domain::security_event::{NewSecurityEvent, SecurityEventType, SecuritySeverity},
    infrastructure::repositories::chained_audit_repository,
};
use crate::modules::geo;

use serde_json::{json, Map, Value};
use sqlx::Row;
use std::net::IpAddr;
use uuid::Uuid;

const IMPOSSIBLE_TRAVEL_KMH: f64 = 900.0;

pub async fn record_event(pool: &sqlx::PgPool, mut event: NewSecurityEvent) {
    enrich_geo_and_travel(pool, &mut event).await;

    if let Err(err) = chained_audit_repository::insert_chained_event(pool, event).await {
        tracing::error!(error = ?err, "failed to write security event");
    }
}

pub async fn record_simple(
    pool: &sqlx::PgPool, user_id: Option<i64>, event_type: SecurityEventType,
    severity: SecuritySeverity, ip_address: Option<IpAddr>,
    user_agent: Option<String>, metadata: Value,
) {
    record_event(pool, NewSecurityEvent {
        user_id, event_type, severity, ip_address, user_agent,
        session_id: None, jti: None, family_id: None, metadata,
    }).await;
}

pub async fn record_session_event(
    pool: &sqlx::PgPool, user_id: Option<i64>, event_type: SecurityEventType,
    severity: SecuritySeverity, ip_address: Option<IpAddr>,
    user_agent: Option<String>, session_id: Option<Uuid>, jti: Option<Uuid>,
    family_id: Option<Uuid>, metadata: Value,
) {
    record_event(pool, NewSecurityEvent {
        user_id, event_type, severity, ip_address, user_agent,
        session_id, jti, family_id, metadata,
    }).await;
}

async fn enrich_geo_and_travel(pool: &sqlx::PgPool, event: &mut NewSecurityEvent) {
    let Some(ip) = event.ip_address else { return; };
    let geo = geo::lookup(ip);
    let mut meta = object_metadata(std::mem::take(&mut event.metadata));

    meta.insert("geoip".to_string(), json!({
        "ip": geo.ip,
        "country": geo.country.clone(),
        "city": geo.city.clone(),
        "latitude": geo.latitude,
        "longitude": geo.longitude,
        "asn": geo.asn.clone(),
        "network_type": geo.network_type.clone(),
    }));

    if let Some(user_id) = event.user_id {
        if let Ok(Some(prev)) = previous_geo_login(pool, user_id).await {
            let elapsed_hours = (chrono::Utc::now() - prev.created_at).num_seconds().max(1) as f64 / 3600.0;
            let distance_km = geo::distance_km(prev.latitude, prev.longitude, geo.latitude, geo.longitude);
            let speed_kmh = distance_km / elapsed_hours;
            let impossible = distance_km > 500.0 && speed_kmh > IMPOSSIBLE_TRAVEL_KMH;

            meta.insert("impossible_travel".to_string(), json!({
                "detected": impossible,
                "distance_km": (distance_km * 10.0).round() / 10.0,
                "elapsed_hours": (elapsed_hours * 100.0).round() / 100.0,
                "speed_kmh": (speed_kmh * 10.0).round() / 10.0,
                "threshold_kmh": IMPOSSIBLE_TRAVEL_KMH,
                "from": {
                    "country": prev.country,
                    "city": prev.city,
                    "latitude": prev.latitude,
                    "longitude": prev.longitude,
                    "at": prev.created_at,
                },
                "to": {
                    "country": geo.country,
                    "city": geo.city,
                    "latitude": geo.latitude,
                    "longitude": geo.longitude,
                }
            }));

            if impossible {
                event.severity = SecuritySeverity::Critical;
            }
        }
    }

    event.metadata = Value::Object(meta);
}

fn object_metadata(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        other if other.is_null() => Map::new(),
        other => {
            let mut map = Map::new();
            map.insert("original_metadata".to_string(), other);
            map
        }
    }
}

struct PreviousGeoLogin {
    created_at: chrono::DateTime<chrono::Utc>,
    country: String,
    city: String,
    latitude: f64,
    longitude: f64,
}

async fn previous_geo_login(pool: &sqlx::PgPool, user_id: i64) -> Result<Option<PreviousGeoLogin>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT created_at, metadata
        FROM security_events
        WHERE user_id = $1
          AND metadata ? 'geoip'
          AND event_type IN ('LOGIN_SUCCESS', 'MFA_SUCCESS', 'REFRESH_SUCCESS')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None); };
    let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
    let metadata: Value = row.try_get("metadata")?;
    let Some(g) = metadata.get("geoip") else { return Ok(None); };

    Ok(Some(PreviousGeoLogin {
        created_at,
        country: g.get("country").and_then(Value::as_str).unwrap_or("UNKNOWN").to_string(),
        city: g.get("city").and_then(Value::as_str).unwrap_or("Unknown").to_string(),
        latitude: g.get("latitude").and_then(Value::as_f64).unwrap_or(0.0),
        longitude: g.get("longitude").and_then(Value::as_f64).unwrap_or(0.0),
    }))
}
