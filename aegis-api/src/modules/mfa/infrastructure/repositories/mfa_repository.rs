use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// MODEL
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserMfa {
    pub id: Uuid,
    pub user_id: i64,

    pub secret: String,
    pub enabled: bool,

    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub disabled_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// FIND
// ---------------------------------------------------------------------------

pub async fn find_by_user_id(
    pool: &sqlx::PgPool,
    user_id: i64,
) -> Result<Option<UserMfa>, sqlx::Error> {
    sqlx::query_as::<_, UserMfa>(
        r#"
        SELECT
            id,
            user_id,
            secret,
            enabled,
            created_at,
            verified_at,
            disabled_at
        FROM user_mfa
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

// ---------------------------------------------------------------------------
// CREATE
// ---------------------------------------------------------------------------

pub async fn create_mfa_secret(
    pool: &sqlx::PgPool,
    user_id: i64,
    secret: &str,
) -> Result<UserMfa, sqlx::Error> {
    sqlx::query_as::<_, UserMfa>(
        r#"
        INSERT INTO user_mfa (
            user_id,
            secret,
            enabled
        )
        VALUES (
            $1,
            $2,
            false
        )
        RETURNING
            id,
            user_id,
            secret,
            enabled,
            created_at,
            verified_at,
            disabled_at
        "#,
    )
    .bind(user_id)
    .bind(secret)
    .fetch_one(pool)
    .await
}

// ---------------------------------------------------------------------------
// ENABLE
// ---------------------------------------------------------------------------

pub async fn enable_mfa(pool: &sqlx::PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE user_mfa
        SET
            enabled = true,
            verified_at = now()
        WHERE user_id = $1
          AND enabled = false
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// DISABLE
// ---------------------------------------------------------------------------

pub async fn disable_mfa(pool: &sqlx::PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE user_mfa
        SET
            enabled = false,
            disabled_at = now()
        WHERE user_id = $1
          AND enabled = true
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
