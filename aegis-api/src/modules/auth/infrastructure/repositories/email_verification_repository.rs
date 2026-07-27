//! Persistence for email verification tokens.
//!
//! Same hashed/single-use/short-TTL pattern as `password_reset_repository`. The
//! raw token is delivered out-of-band (email); only its SHA-256 is stored.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

pub async fn insert_token(
    pool: &PgPool,
    user_id: i64,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO email_verification_tokens (user_id, token_hash, expires_at)
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

pub async fn find_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<EmailVerificationToken>, sqlx::Error> {
    sqlx::query_as::<_, EmailVerificationToken>(
        r#"
        SELECT id, user_id, token_hash, created_at, expires_at, used_at
        FROM email_verification_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

pub async fn mark_used(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE email_verification_tokens
        SET used_at = now()
        WHERE id = $1 AND used_at IS NULL
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn invalidate_user_tokens(pool: &PgPool, user_id: i64) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE email_verification_tokens
        SET used_at = now()
        WHERE user_id = $1 AND used_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

/// Mark the user as verified.
pub async fn set_user_verified(pool: &PgPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET email_verified_at = now()
        WHERE id = $1
          AND email_verified_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}
