pub mod login;
pub mod origins;
pub mod travel;

use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, Serialize)]
pub struct GeoIpInfo {
    pub ip: String,
    pub country: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub asn: String,
    pub network_type: String,
}

/// Offline GeoIP fallback. Production can replace this with maxminddb without
/// touching dashboard code because the metadata contract stays stable.
pub fn lookup(ip: IpAddr) -> GeoIpInfo {
    if ip.is_loopback() {
        return info(ip, "LOCAL", "Loopback", 0.0, 0.0, "AS0 Local", "loopback");
    }
    if is_private(ip) {
        return info(ip, "PRIVATE", "RFC1918/RFC4193", 0.0, 0.0, "AS0 Private", "private");
    }
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            match o[0] {
                8 => info(ip, "US", "Mountain View", 37.4056, -122.0775, "AS15169 Google", "public"),
                1 => info(ip, "AU", "Sydney", -33.8688, 151.2093, "AS13335 Cloudflare", "public"),
                31 => info(ip, "ES", "Madrid", 40.4168, -3.7038, "AS3352 Telefonica", "public"),
                45 => info(ip, "BR", "Sao Paulo", -23.5505, -46.6333, "AS0 Public", "public"),
                80..=95 => info(ip, "DE", "Frankfurt", 50.1109, 8.6821, "AS3320 Deutsche Telekom", "public"),
                100..=126 => info(ip, "US", "New York", 40.7128, -74.0060, "AS0 Public", "public"),
                128..=159 => info(ip, "JP", "Tokyo", 35.6762, 139.6503, "AS0 Public", "public"),
                160..=191 => info(ip, "GB", "London", 51.5072, -0.1276, "AS0 Public", "public"),
                _ => info(ip, "US", "Ashburn", 39.0438, -77.4874, "AS0 Public", "public"),
            }
        }
        IpAddr::V6(_) => info(ip, "US", "Ashburn", 39.0438, -77.4874, "AS0 IPv6", "public"),
    }
}

fn info(ip: IpAddr, country: &str, city: &str, lat: f64, lon: f64, asn: &str, kind: &str) -> GeoIpInfo {
    GeoIpInfo { ip: ip.to_string(), country: country.into(), city: city.into(), latitude: lat, longitude: lon, asn: asn.into(), network_type: kind.into() }
}

fn is_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_private() || v4.is_link_local() || v4 == Ipv4Addr::new(0,0,0,0),
        IpAddr::V6(v6) => v6.is_unique_local() || v6.is_unicast_link_local() || v6 == Ipv6Addr::UNSPECIFIED,
    }
}

pub fn distance_km(a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    let r = 6371.0_f64;
    let dlat = (b_lat - a_lat).to_radians();
    let dlon = (b_lon - a_lon).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + a_lat.to_radians().cos() * b_lat.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}
