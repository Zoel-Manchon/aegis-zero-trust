use crate::modules::auth::domain::session::session_status::SessionStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    /// Immutable DB identity.
    pub id: Uuid,

    /// Refresh-token family lineage.
    /// All rotated refresh tokens from the same login share this value.
    pub family_id: Uuid,

    /// User who owns the session.
    pub user_id: i64,

    /// HMAC hash of the currently valid refresh token.
    pub refresh_token_hash: String,

    /// JWT ID bound to the current refresh token.
    /// This changes on every refresh rotation.
    pub jti: Uuid,

    /// Human-readable device metadata.
    /// Do not treat this as a trusted security primitive.
    pub device_name: String,

    /// Initial session IP address.
    /// Use as a risk signal, not as hard identity.
    pub ip_address: IpAddr,

    /// Initial User-Agent.
    /// Weak continuity signal only.
    pub user_agent: String,

    /// Session lifecycle state.
    pub status: SessionStatus,

    /// Previous session row ID/JTI depending on your repository design.
    /// Be consistent: if this points to old `jti`, name it `rotated_from_jti`.
    /// If it points to old `id`, name it `rotated_from_session_id`.
    pub rotated_from: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
