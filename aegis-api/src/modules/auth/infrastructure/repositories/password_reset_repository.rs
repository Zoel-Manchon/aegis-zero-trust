//! Persistence for password reset tokens.
//!
//! Follows the existing free-function repository style used elsewhere in auth
//! (e.g. `session_repository`). All functions take `&PgPool` and return
//! `sqlx::Error` on failure; the service layer maps those to `AppError`.
//!
//! Security note: this layer only ever sees the *hash* of a token. The raw
//! token exists briefly in the service layer (to build the reset link) and is
//! never persisted.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// A stored reset-token row (hash only).
#[derive(Debug, sqlx::FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

/// Insert a new reset token (hash + expiry) for a user.
pub async fn insert_token(
    pool: &PgPool,
    user_id: i64,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Look up a token by its hash. Returns the row even if expired/used so the
/// service can decide and log precisely; the service is responsible for the
/// validity checks.
pub async fn find_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<PasswordResetToken>, sqlx::Error> {
    sqlx::query_as::<_, PasswordResetToken>(
        r#"
        SELECT id, user_id, token_hash, created_at, expires_at, used_at
        FROM password_reset_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

/// Mark a token as used (single-use enforcement). Returns true if a row was
/// updated (i.e. it was still unused).
pub async fn mark_used(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE password_reset_tokens
        SET used_at = now()
        WHERE id = $1
          AND used_at IS NULL
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected() > 0)
}

/// Invalidate all outstanding (unused, unexpired) tokens for a user. Called
/// after a successful reset and whenever a fresh token is requested, so only
/// the newest link is ever live.
pub async fn invalidate_user_tokens(pool: &PgPool, user_id: i64) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE password_reset_tokens
        SET used_at = now()
        WHERE user_id = $1
          AND used_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}
