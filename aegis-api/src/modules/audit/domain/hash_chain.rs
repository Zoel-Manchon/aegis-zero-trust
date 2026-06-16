//! Hash-chain primitives for the tamper-evident audit log.

use sha2::{Digest, Sha256};

pub const GENESIS_PREV_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

pub fn canonical_payload(
    seq: i64, prev_hash: &str, user_id: Option<i64>, event_type: &str,
    severity: &str, ip_address: Option<&str>, session_id: Option<&str>,
    jti: Option<&str>, family_id: Option<&str>, metadata_json: &str,
    created_at_rfc3339: &str,
) -> String {
    format!(
        "{seq}|{prev}|{uid}|{et}|{sev}|{ip}|{sid}|{jti}|{fid}|{meta}|{ts}",
        seq = seq, prev = prev_hash,
        uid = user_id.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
        et = event_type, sev = severity,
        ip = ip_address.unwrap_or("-"), sid = session_id.unwrap_or("-"),
        jti = jti.unwrap_or("-"), fid = family_id.unwrap_or("-"),
        meta = metadata_json, ts = created_at_rfc3339,
    )
}

pub fn hash_payload(canonical: &str) -> String {
    hex::encode(Sha256::digest(canonical.as_bytes()))
}

/// Deterministically serialize a JSON value with sorted object keys.
///
/// CRITICAL: Postgres `jsonb` does NOT preserve key order or whitespace — it
/// normalizes to a binary form and reserializes on read. So the metadata string
/// we hash at INSERT time (`value.to_string()`) will not match the string we get
/// back at VERIFY time. Hashing this canonical (key-sorted) form on BOTH paths
/// makes the metadata hash stable across the jsonb round-trip.
pub fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let inner: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", serde_json::to_string(k).unwrap(), canonical_json(&map[k])))
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        serde_json::Value::Array(arr) => {
            let inner: Vec<String> = arr.iter().map(canonical_json).collect();
            format!("[{}]", inner.join(","))
        }
        // Scalars serialize deterministically already.
        other => other.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn compute_event_hash(
    seq: i64, prev_hash: &str, user_id: Option<i64>, event_type: &str,
    severity: &str, ip_address: Option<&str>, session_id: Option<&str>,
    jti: Option<&str>, family_id: Option<&str>, metadata_json: &str,
    created_at_rfc3339: &str,
) -> String {
    hash_payload(&canonical_payload(
        seq, prev_hash, user_id, event_type, severity, ip_address,
        session_id, jti, family_id, metadata_json, created_at_rfc3339,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashing_is_deterministic() {
        let a = compute_event_hash(1, GENESIS_PREV_HASH, Some(1), "LOGIN_SUCCESS", "INFO",
            Some("127.0.0.1"), None, None, None, "{}", "2026-01-01T00:00:00Z");
        let b = compute_event_hash(1, GENESIS_PREV_HASH, Some(1), "LOGIN_SUCCESS", "INFO",
            Some("127.0.0.1"), None, None, None, "{}", "2026-01-01T00:00:00Z");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn changing_any_field_changes_hash() {
        let base = compute_event_hash(1, GENESIS_PREV_HASH, Some(1), "LOGIN_SUCCESS", "INFO",
            Some("127.0.0.1"), None, None, None, "{}", "2026-01-01T00:00:00Z");
        let tampered = compute_event_hash(1, GENESIS_PREV_HASH, Some(1), "LOGIN_SUCCESS", "CRITICAL",
            Some("127.0.0.1"), None, None, None, "{}", "2026-01-01T00:00:00Z");
        assert_ne!(base, tampered);
    }

    #[test]
    fn prev_hash_binds_the_chain() {
        let h1 = compute_event_hash(1, GENESIS_PREV_HASH, None, "X", "INFO", None, None, None, None, "{}", "t");
        let from_h1 = compute_event_hash(2, &h1, None, "Y", "INFO", None, None, None, None, "{}", "t");
        let from_gen = compute_event_hash(2, GENESIS_PREV_HASH, None, "Y", "INFO", None, None, None, None, "{}", "t");
        assert_ne!(from_h1, from_gen);
    }
}
