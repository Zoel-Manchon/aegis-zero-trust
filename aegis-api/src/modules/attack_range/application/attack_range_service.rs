//! The attack-range engine: resolve an origin, write a scenario's worth of
//! attributed security events, run impossible-travel detection against the
//! victim's last sighting, and dispatch alerts (which reach the SOC over the
//! WebSocket broadcast bus).

use crate::app_state::AppState;
use crate::core::errors::app_error::AppError;
use crate::modules::alerts::domain::alert::{Alert, AlertSeverity};
use crate::modules::audit::application::audit_service;
use crate::modules::audit::domain::security_event::{SecurityEventType, SecuritySeverity};
use crate::modules::auth::infrastructure::repositories::user_repository::UserRepository;
use crate::modules::geo::{self, origins, travel::GeoPoint};

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct LaunchRequest {
    /// Scenario key (see `scenarios()`): brute_force | credential_stuffing |
    /// token_replay | jwt_tamper | fingerprint_spoof | session_hijack |
    /// mfa_bypass | rbac_bypass | privilege_escalation | storm.
    pub scenario: String,
    /// Attacker origin: a preset key (e.g. "madrid") or a raw public IP.
    pub origin: String,
    /// The account being attacked (its last location is tracked for travel).
    pub victim_email: String,
}

#[derive(Debug, Serialize)]
pub struct LaunchReport {
    pub scenario: String,
    pub origin_ip: String,
    pub origin: GeoPoint,
    pub events_recorded: usize,
    pub impossible_travel: bool,
    pub distance_km: f64,
    pub speed_kmh: f64,
    pub from: Option<GeoPoint>,
}

#[derive(Debug, Serialize)]
pub struct ScenarioInfo {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

pub fn scenarios() -> Vec<ScenarioInfo> {
    vec![
        ScenarioInfo { key: "brute_force", label: "Brute force", description: "Repeated failed logins until lockout." },
        ScenarioInfo { key: "credential_stuffing", label: "Credential stuffing", description: "High-volume reused-credential logins." },
        ScenarioInfo { key: "token_replay", label: "Refresh replay", description: "Reuse a rotated refresh token." },
        ScenarioInfo { key: "jwt_tamper", label: "JWT tamper", description: "Forged/tampered access token." },
        ScenarioInfo { key: "fingerprint_spoof", label: "Fingerprint spoof", description: "Forged device fingerprint to defeat binding." },
        ScenarioInfo { key: "session_hijack", label: "Session hijack", description: "Stolen session token replayed from a new device." },
        ScenarioInfo { key: "mfa_bypass", label: "MFA bypass", description: "Step-up / MFA failures probing for a gap." },
        ScenarioInfo { key: "rbac_bypass", label: "RBAC bypass", description: "Normal user reaching an admin route." },
        ScenarioInfo { key: "privilege_escalation", label: "Privilege escalation", description: "User attempts to elevate to admin." },
        ScenarioInfo { key: "storm", label: "⚡ Storm (multi-vector)", description: "Rapid burst across many vectors at once." },
    ]
}

fn scenario_events(scenario: &str) -> Vec<(SecurityEventType, SecuritySeverity)> {
    use SecurityEventType::*;
    use SecuritySeverity::*;
    match scenario {
        "brute_force" => vec![
            (LoginFailure, Medium),
            (LoginFailure, Medium),
            (LoginFailure, High),
            (LoginFailure, High),
            (BruteForceLockout, High),
        ],
        "credential_stuffing" => vec![
            (LoginFailure, Medium),
            (LoginFailure, Medium),
            (LoginFailure, Medium),
            (LoginFailure, High),
            (CredentialStuffing, High),
            (BruteForceLockout, High),
        ],
        "token_replay" => vec![(LoginSuccess, Info), (RefreshReplayDetected, Critical)],
        "jwt_tamper" => vec![(TokenPurposeViolation, High)],
        "fingerprint_spoof" => vec![
            (LoginSuccess, Info),
            (DeviceFingerprintMismatch, High),
            (PolicyDenied, High),
            (SessionRevoked, Medium),
        ],
        "session_hijack" => vec![
            (SessionHijack, Critical),
            (TokenPurposeViolation, High),
            (SessionRevoked, Medium),
        ],
        "mfa_bypass" => vec![
            (MfaRequired, Info),
            (MfaFailure, Medium),
            (MfaFailure, High),
            (PolicyDenied, High),
        ],
        "rbac_bypass" => vec![(PolicyDenied, High)],
        "privilege_escalation" => vec![
            (PolicyDenied, High),
            (PrivilegeEscalation, Critical),
            (SessionRevoked, Medium),
        ],
        // A multi-vector burst — one launch writes a wave of events.
        "storm" => vec![
            (LoginFailure, Medium),
            (LoginFailure, Medium),
            (LoginFailure, High),
            (CredentialStuffing, High),
            (BruteForceLockout, High),
            (DeviceFingerprintMismatch, High),
            (TokenPurposeViolation, High),
            (RefreshReplayDetected, Critical),
            (PolicyDenied, High),
            (SessionHijack, Critical),
            (PrivilegeEscalation, Critical),
            (SessionRevoked, Medium),
        ],
        _ => vec![(LoginFailure, Low)],
    }
}

fn scenario_label(scenario: &str) -> &'static str {
    scenarios()
        .into_iter()
        .find(|s| s.key == scenario)
        .map(|s| s.label)
        .unwrap_or("Recon")
}

pub async fn launch(state: &AppState, req: LaunchRequest) -> Result<LaunchReport, AppError> {
    let LaunchRequest { scenario, origin, victim_email } = req;

    // Resolve the origin (preset key or raw IP) into a geo-locatable address.
    let ip_str = origins::resolve(&origin);
    let ip: IpAddr = ip_str.parse().map_err(|_| AppError::BadRequest)?;
    let geo = geo::lookup(ip);

    // The victim must exist so impossible-travel can track a per-user location.
    let victim = UserRepository::find_by_email(&state.pool, &victim_email)
        .await
        .map_err(|_| AppError::DatabaseError)?
        .ok_or(AppError::NotFound)?;
    let victim_id = Some(victim.id);

    let label = scenario_label(&scenario);
    // The frontend's derive.ts reads `metadata.geoip` (city/country/lat/lon/
    // network_type) and `metadata.impossible_travel.detected`. Match that shape.
    let geoip = json!({
        "ip": geo.ip.as_str(),
        "country": geo.country.as_str(),
        "city": geo.city.as_str(),
        "latitude": geo.latitude,
        "longitude": geo.longitude,
        "network_type": geo.network_type.as_str(),
        "asn": geo.asn.as_str(),
    });
    let base_meta = json!({
        "source": "attack-range",
        "scenario": scenario.as_str(),
        "origin_ip": ip_str.as_str(),
        "geoip": geoip.clone(),
    });

    // 1. Write the scenario's events (each NOTIFYs → live event feed).
    let mut recorded = 0usize;
    for (event_type, severity) in scenario_events(&scenario) {
        audit_service::record_session_event(
            &state.pool,
            victim_id,
            event_type,
            severity,
            Some(ip),
            Some("attack-range-simulator".to_string()),
            None,
            None,
            None,
            base_meta.clone(),
        )
        .await;
        recorded += 1;
    }

    // 2. A scenario alert — reaches the SOC popup over the WS bus immediately.
    state
        .alerts
        .dispatch(
            &Alert::new(
                scenario.clone(),
                AlertSeverity::High,
                format!("{label} attack from {}, {}", geo.city, geo.country),
                format!("Scenario '{scenario}' launched from {ip_str} ({}).", geo.country),
            )
            .with_meta("city", geo.city.clone())
            .with_meta("country", geo.country.clone())
            .with_meta("ip", ip_str.clone()),
        )
        .await;

    // 3. Impossible-travel check (skip placeholder 0,0 for private/loopback).
    let mut impossible = false;
    let mut distance_km = 0.0;
    let mut speed_kmh = 0.0;
    let mut from: Option<GeoPoint> = None;

    if geo.latitude != 0.0 || geo.longitude != 0.0 {
        let verdict = geo::travel::evaluate(&state.redis, victim.id, &geo).await;
        distance_km = verdict.distance_km;
        speed_kmh = verdict.speed_kmh;
        from = verdict.from.clone();
        impossible = verdict.impossible;

        if impossible {
            let prior = from.clone().unwrap_or(GeoPoint {
                country: "?".into(),
                city: "?".into(),
                lat: 0.0,
                lon: 0.0,
            });
            let travel_meta = json!({
                "source": "attack-range",
                "scenario": scenario.as_str(),
                "origin_ip": ip_str.as_str(),
                "geoip": geoip.clone(),
                "impossible_travel": {
                    "detected": true,
                    "distance_km": distance_km.round(),
                    "speed_kmh": speed_kmh.round(),
                    "from": { "country": prior.country.as_str(), "city": prior.city.as_str() },
                    "to": { "country": geo.country.as_str(), "city": geo.city.as_str() },
                },
            });
            audit_service::record_session_event(
                &state.pool,
                victim_id,
                SecurityEventType::ImpossibleTravel,
                SecuritySeverity::Critical,
                Some(ip),
                Some("attack-range-simulator".to_string()),
                None,
                None,
                None,
                travel_meta,
            )
            .await;
            recorded += 1;

            state
                .alerts
                .dispatch(
                    &Alert::new(
                        "impossible_travel",
                        AlertSeverity::Critical,
                        format!("Impossible travel: {} → {}", prior.city, geo.city),
                        format!(
                            "{} → {} ({} km) in moments — {} km/h. Account: {}.",
                            prior.city,
                            geo.city,
                            distance_km.round(),
                            speed_kmh.round(),
                            victim.email,
                        ),
                    )
                    .with_meta("from_city", prior.city.clone())
                    .with_meta("to_city", geo.city.clone())
                    .with_meta("distance_km", distance_km.round().to_string())
                    .with_meta("speed_kmh", speed_kmh.round().to_string()),
                )
                .await;
        }
    }

    Ok(LaunchReport {
        scenario,
        origin_ip: ip_str,
        origin: GeoPoint {
            country: geo.country,
            city: geo.city,
            lat: geo.latitude,
            lon: geo.longitude,
        },
        events_recorded: recorded,
        impossible_travel: impossible,
        distance_km,
        speed_kmh,
        from,
    })
}
