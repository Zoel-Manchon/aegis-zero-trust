use crate::modules::auth::models::user_model::UserRole;
use uuid::Uuid;
pub struct AuthUser {
    pub id: i64,
    pub role: UserRole,
    pub jti: Uuid,
}
