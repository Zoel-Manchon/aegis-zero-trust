//! Real-login geolocation.
//!
//! On a successful login, geo-locate the client IP, run impossible-travel
//! detection (recording a critical event + alert if tripped), and hand back the
//! `geoip` metadata to embed in the login event so the SOC map shows where the
//! user signed in from. Loopback/private origins (lat/lon 0,0) are skipped.

use crate::app_state::AppState;
use crate::modules::alerts::domain::alert::{Alert, AlertSeverity};
use crate::modules::audit::application::audit_service;
use crate::modules::audit::domain::security_event::{SecurityEventType, SecuritySeverity};
use crate::modules::geo::{self, travel::GeoPoint};

use serde_json::{Value, json};
use std::net::IpAddr;

pub async fn record_login_geo(
    state: &AppState,
    user_id: i64,
    ip: IpAddr,
    user_agent: Option<String>,
) -> Option<Value> {
    let geo = geo::lookup(ip);
    if geo.latitude == 0.0 && geo.longitude == 0.0 {
        return None; // loopback / private — no real location
    }

    let geoip = json!({
        "ip": geo.ip.as_str(),
        "country": geo.country.as_str(),
        "city": geo.city.as_str(),
        "latitude": geo.latitude,
        "longitude": geo.longitude,
        "network_type": geo.network_type.as_str(),
        "asn": geo.asn.as_str(),
    });

    let verdict = geo::travel::evaluate(&state.redis, user_id, &geo).await;
    if verdict.impossible {
        let prior = verdict.from.clone().unwrap_or(GeoPoint {
            country: "?".into(),
            city: "?".into(),
            lat: 0.0,
            lon: 0.0,
        });
        let meta = json!({
            "source": "login",
            "geoip": geoip.clone(),
            "impossible_travel": {
                "detected": true,
                "distance_km": verdict.distance_km.round(),
                "speed_kmh": verdict.speed_kmh.round(),
                "from": { "country": prior.country.as_str(), "city": prior.city.as_str() },
                "to": { "country": geo.country.as_str(), "city": geo.city.as_str() },
            },
        });
        audit_service::record_session_event(
            &state.pool,
            Some(user_id),
            SecurityEventType::ImpossibleTravel,
            SecuritySeverity::Critical,
            Some(ip),
            user_agent,
            None,
            None,
            None,
            meta,
        )
        .await;

        state
            .alerts
            .dispatch(
                &Alert::new(
                    "impossible_travel",
                    AlertSeverity::Critical,
                    format!("Impossible travel: {} → {}", prior.city, geo.city),
                    format!(
                        "Login for user {user_id} jumped {} → {} ({} km) — {} km/h.",
                        prior.city,
                        geo.city,
                        verdict.distance_km.round(),
                        verdict.speed_kmh.round(),
                    ),
                )
                .with_meta("from_city", prior.city.clone())
                .with_meta("to_city", geo.city.clone())
                .with_meta("distance_km", verdict.distance_km.round().to_string())
                .with_meta("speed_kmh", verdict.speed_kmh.round().to_string()),
            )
            .await;
    }

    Some(geoip)
}
