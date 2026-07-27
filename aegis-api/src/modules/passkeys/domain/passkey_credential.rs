use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Persisted WebAuthn/passkey credential metadata.
///
/// Zero Trust rule: this table must never store a private key or a reusable
/// secret. The authenticator keeps the private key; the server stores only the
/// credential id, public key, signature counter, and device metadata used for
/// risk decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    pub id: i64,
    pub user_id: i64,
    pub credential_id: String,
    pub public_key_cose: Vec<u8>,
    pub sign_count: i64,
    pub friendly_name: Option<String>,
    pub transports: Vec<String>,
    pub aaguid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}
