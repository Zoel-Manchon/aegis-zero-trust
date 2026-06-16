use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SecurityEventView {
    pub id: Uuid,
    pub user_id: Option<i64>,
    pub event_type: String,
    pub severity: String,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub session_id: Option<Uuid>,
    pub jti: Option<Uuid>,
    pub family_id: Option<Uuid>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
