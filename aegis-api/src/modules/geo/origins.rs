//! Attacker-origin presets for the SOC "launch attack" control.
//!
//! Each preset pairs a human label with a representative public IP whose octets
//! resolve (via `geo::lookup`) to that city's coordinates. The dashboard shows
//! the labels; the backend geo-locates the IP. Launching from two distant
//! presets in a row is what trips impossible-travel detection.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Origin {
    pub key: &'static str,
    pub label: &'static str,
    pub ip: &'static str,
}

pub fn presets() -> Vec<Origin> {
    vec![
        Origin { key: "madrid", label: "Madrid, ES", ip: "31.10.10.10" },
        Origin { key: "tokyo", label: "Tokyo, JP", ip: "133.10.10.10" },
        Origin { key: "new_york", label: "New York, US", ip: "104.10.10.10" },
        Origin { key: "london", label: "London, GB", ip: "165.10.10.10" },
        Origin { key: "frankfurt", label: "Frankfurt, DE", ip: "85.10.10.10" },
        Origin { key: "sydney", label: "Sydney, AU", ip: "1.10.10.10" },
        Origin { key: "sao_paulo", label: "Sao Paulo, BR", ip: "45.10.10.10" },
    ]
}

/// Resolve an origin selector value (a preset key or a raw IP) to an IP string.
pub fn resolve(value: &str) -> String {
    presets()
        .into_iter()
        .find(|o| o.key == value)
        .map(|o| o.ip.to_string())
        .unwrap_or_else(|| value.to_string())
}
