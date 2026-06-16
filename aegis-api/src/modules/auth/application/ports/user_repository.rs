//! Port: user persistence.
//!
//! The application layer (auth_service, future password-recovery service) depends
//! on this trait, not on a concrete Postgres type. The adapter in
//! `infrastructure/` implements it.
//!
//! Errors are mapped to the domain `AppError` here so nothing above this line
//! ever sees `sqlx::Error` — the storage technology stays an implementation
//! detail.

use crate::core::errors::app_error::AppError;
use crate::modules::auth::models::user_model::User;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Look up a user by email (case-sensitive match on the stored value).
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;

    /// Look up a user by primary key.
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError>;

    /// Create a user with an already-hashed password. Returns the persisted row.
    async fn create_user(&self, email: &str, hashed_password: &str) -> Result<User, AppError>;
}
pub async fn update_password(
        pool: &PgPool,
        user_id: i64,
        new_password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET password = $2
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(new_password_hash)
        .execute(pool)
        .await?;

        Ok(())
    }