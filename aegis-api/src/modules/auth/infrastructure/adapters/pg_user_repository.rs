//! Postgres adapter for the `UserRepository` port.
//!
//! A thin wrapper over the existing `UserRepository` inherent methods (kept in
//! infrastructure/repositories). We deliberately do NOT rewrite the SQL — it's
//! already correct and tested — we just adapt it to the trait and let the
//! `From<sqlx::Error> for AppError` impl map errors via `?`.

use crate::core::errors::app_error::AppError;
use crate::modules::auth::application::ports::user_repository::UserRepository as UserRepositoryPort;
use crate::modules::auth::infrastructure::repositories::user_repository::UserRepository as UserRepoSql;
use crate::modules::auth::models::user_model::User;
use async_trait::async_trait;
use sqlx::PgPool;

/// Holds the shared pool. Cheap to clone.
#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryPort for PgUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        Ok(UserRepoSql::find_by_email(&self.pool, email).await?)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        Ok(UserRepoSql::find_by_id(&self.pool, id).await?)
    }

    async fn create_user(&self, email: &str, hashed_password: &str) -> Result<User, AppError> {
        Ok(UserRepoSql::create_user(&self.pool, email, hashed_password).await?)
    }
}
