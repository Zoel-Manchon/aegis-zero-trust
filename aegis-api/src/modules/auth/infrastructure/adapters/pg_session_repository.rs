//! Postgres adapter for the `SessionRepository` port.
//!
//! Thin wrapper over the existing free functions in
//! `infrastructure/repositories/session_repository`. The SQL (including the
//! atomic rotation transaction) is unchanged; we only adapt it to the trait and
//! map errors to `AppError` via `?`.

use crate::core::errors::app_error::AppError;
use crate::modules::auth::application::ports::session_repository::SessionRepository as SessionRepositoryPort;
use crate::modules::auth::domain::session::new_session::NewSession;
use crate::modules::auth::domain::session::session::Session;
use crate::modules::auth::infrastructure::repositories::session_repository as repo;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Holds the shared pool. Cheap to clone.
#[derive(Clone)]
pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepositoryPort for PgSessionRepository {
    async fn insert_session(&self, session: NewSession) -> Result<Session, AppError> {
        Ok(repo::insert_session(&self.pool, session).await?)
    }

    async fn find_valid_session_by_jti(&self, jti: Uuid) -> Result<Option<Session>, AppError> {
        Ok(repo::find_valid_session_by_jti(&self.pool, jti).await?)
    }

    async fn find_by_jti_raw(&self, jti: Uuid) -> Result<Option<Session>, AppError> {
        Ok(repo::find_by_jti_raw(&self.pool, jti).await?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>, AppError> {
        Ok(repo::find_by_id(&self.pool, id).await?)
    }

    async fn revoke_session(&self, jti: Uuid) -> Result<bool, AppError> {
        Ok(repo::revoke_session(&self.pool, jti).await?)
    }

    async fn revoke_all_user_sessions(&self, user_id: i64) -> Result<u64, AppError> {
        Ok(repo::revoke_all_user_sessions(&self.pool, user_id).await?)
    }

    async fn revoke_family(&self, family_id: Uuid) -> Result<u64, AppError> {
        Ok(repo::revoke_family(&self.pool, family_id).await?)
    }

    async fn rotate_session_atomic(
        &self,
        session_id: Uuid,
        new_jti: Uuid,
        new_hash: String,
    ) -> Result<Option<Session>, AppError> {
        Ok(repo::rotate_session_atomic(&self.pool, session_id, new_jti, new_hash).await?)
    }

    async fn touch_session(&self, jti: Uuid) -> Result<(), AppError> {
        Ok(repo::touch_session(&self.pool, jti).await?)
    }
}
