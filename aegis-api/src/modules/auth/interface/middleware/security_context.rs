use crate::modules::auth::models::user_model::UserRole;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SecurityContext {
    pub user_id: i64,
    pub role: UserRole,
    pub jti: Uuid,
    pub session_id: Uuid,

    pub ip: IpAddr,
    pub user_agent: String,

    pub risk_score: u8,
}
