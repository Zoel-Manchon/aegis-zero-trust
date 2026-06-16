//! Port: session persistence.
//!
//! Covers the full session lifecycle the auth/refresh services need: insert,
//! lookup (valid / raw / by-id), revoke (one / all-for-user / whole-family),
//! atomic rotation, and touch. Mirrors the existing `session_repository`
//! functions exactly so the adapter is a thin wrapper.
//!
//! As with the user port, errors surface as `AppError`, not `sqlx::Error`.

use crate::core::errors::app_error::AppError;
use crate::modules::auth::domain::session::new_session::NewSession;
use crate::modules::auth::domain::session::session::Session;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Insert a fresh session (status defaults to 'active', 7-day expiry).
    async fn insert_session(&self, session: NewSession) -> Result<Session, AppError>;

    /// Find an active, unexpired session by its JWT id.
    async fn find_valid_session_by_jti(&self, jti: Uuid) -> Result<Option<Session>, AppError>;

    /// Find a session by JWT id regardless of status/expiry (for reuse detection).
    async fn find_by_jti_raw(&self, jti: Uuid) -> Result<Option<Session>, AppError>;

    /// Find a session by primary key.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>, AppError>;

    /// Revoke a single active session by JWT id. Returns true if one was revoked.
    async fn revoke_session(&self, jti: Uuid) -> Result<bool, AppError>;

    /// Revoke all active sessions for a user. Returns the count revoked.
    async fn revoke_all_user_sessions(&self, user_id: i64) -> Result<u64, AppError>;

    /// Revoke every active session in a refresh-token family (reuse response).
    async fn revoke_family(&self, family_id: Uuid) -> Result<u64, AppError>;

    /// Atomically rotate: mark the old session 'rotated' and insert its
    /// successor in one transaction. Returns the new session, or None if the old
    /// one was not active/valid.
    async fn rotate_session_atomic(
        &self,
        session_id: Uuid,
        new_jti: Uuid,
        new_hash: String,
    ) -> Result<Option<Session>, AppError>;

    /// Bump `last_used_at` for an active session.
    async fn touch_session(&self, jti: Uuid) -> Result<(), AppError>;
}
