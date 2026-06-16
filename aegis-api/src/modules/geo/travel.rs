//! Impossible-travel detection.
//!
//! Ports the reference rule: if a user's location moves more than 100 km between
//! two sightings and the implied speed exceeds 900 km/h (faster than any
//! commercial flight), it cannot be the same human — flag it.
//!
//! The user's last known location is cached in Redis (`geo:last:{user_id}`), so
//! detection is a single get/compare/set. Loopback/private origins resolve to
//! (0,0) in `geo::lookup` and are skipped by the caller, so local traffic never
//! produces phantom alerts.

use crate::core::cache::redis::RedisClient;
use crate::modules::geo::GeoIpInfo;
use serde::{Deserialize, Serialize};

const LAST_GEO_TTL_SECS: usize = 60 * 60 * 24 * 30; // 30 days
const MIN_DISTANCE_KM: f64 = 100.0;
const MAX_PLAUSIBLE_KMH: f64 = 900.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LastGeo {
    lat: f64,
    lon: f64,
    country: String,
    city: String,
    /// Unix seconds of the sighting.
    ts: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeoPoint {
    pub country: String,
    pub city: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TravelVerdict {
    pub impossible: bool,
    pub distance_km: f64,
    pub speed_kmh: f64,
    pub from: Option<GeoPoint>,
    pub to: GeoPoint,
}

/// Compare `geo` against the user's last sighting, update the cache, and return
/// the verdict. Never errors out the caller: a Redis hiccup just yields a
/// non-impossible verdict with no prior point.
pub async fn evaluate(redis: &RedisClient, user_id: i64, geo: &GeoIpInfo) -> TravelVerdict {
    let key = format!("geo:last:{user_id}");
    let now = chrono::Utc::now().timestamp();

    let prior: Option<LastGeo> = redis
        .get_string(&key)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());

    let to = GeoPoint {
        country: geo.country.clone(),
        city: geo.city.clone(),
        lat: geo.latitude,
        lon: geo.longitude,
    };

    let mut verdict = TravelVerdict {
        impossible: false,
        distance_km: 0.0,
        speed_kmh: 0.0,
        from: None,
        to: to.clone(),
    };

    if let Some(p) = prior {
        let km = super::distance_km(p.lat, p.lon, geo.latitude, geo.longitude);
        let hours = ((now - p.ts).max(1) as f64) / 3600.0;
        let speed = km / hours;
        verdict.distance_km = km;
        verdict.speed_kmh = speed;
        verdict.from = Some(GeoPoint { country: p.country, city: p.city, lat: p.lat, lon: p.lon });
        verdict.impossible = km > MIN_DISTANCE_KM && speed > MAX_PLAUSIBLE_KMH;
    }

    // Update the last sighting regardless, so the *next* event compares to this.
    let current = LastGeo {
        lat: geo.latitude,
        lon: geo.longitude,
        country: geo.country.clone(),
        city: geo.city.clone(),
        ts: now,
    };
    if let Ok(json) = serde_json::to_string(&current) {
        let _ = redis.set_ex(&key, &json, LAST_GEO_TTL_SECS).await;
    }

    verdict
}
