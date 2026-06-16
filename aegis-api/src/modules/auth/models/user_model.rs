use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, sqlx::Type, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

// Clean + idiomatic conversion layer
impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role = match self {
            UserRole::User => "user",
            UserRole::Admin => "admin",
        };
        write!(f, "{}", role)
    }
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub user_role: UserRole,
    pub created_at: DateTime<Utc>,
}
