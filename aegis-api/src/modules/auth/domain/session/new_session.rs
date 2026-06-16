use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NewSession {
    pub user_id: i64,
    pub family_id: Uuid,

    pub refresh_token_hash: String,
    pub jti: Uuid,

    /// Display metadata only.
    pub device_name: String,

    /// Initial request metadata.
    pub ip_address: IpAddr,
    pub user_agent: String,

    /// Previous session reference during refresh rotation.
    /// Should be None for fresh login.
    pub rotated_from: Option<Uuid>,
}
