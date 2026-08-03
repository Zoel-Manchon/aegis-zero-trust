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

    /// Highest TOTP step already spent by this user. `None` until the first
    /// successful verification. See `try_consume_step`.
    pub last_used_step: Option<i64>,
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
            disabled_at,
            last_used_step
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
            disabled_at,
            last_used_step
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

// ---------------------------------------------------------------------------
// REPLAY PREVENTION
// ---------------------------------------------------------------------------

/// Atomically claim a TOTP step for this user.
///
/// Returns `true` only if the step is strictly newer than the last one spent.
/// The comparison and the write happen in a single UPDATE, so two requests
/// racing with the same intercepted code cannot both win: Postgres serializes
/// them on the row lock and the loser sees its own step is no longer greater.
pub async fn try_consume_step(
    pool: &sqlx::PgPool,
    user_id: i64,
    step: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE user_mfa
        SET last_used_step = $2
        WHERE user_id = $1
          AND enabled = true
          AND (last_used_step IS NULL OR last_used_step < $2)
        "#,
    )
    .bind(user_id)
    .bind(step)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// BACKUP CODES
// ---------------------------------------------------------------------------

/// Replace this user's backup codes with a fresh set.
///
/// Old codes are deleted in the same transaction, so a regenerate never leaves
/// a window where both sets work.
pub async fn replace_backup_codes(
    pool: &sqlx::PgPool,
    user_id: i64,
    hashes: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM mfa_backup_codes WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    for hash in hashes {
        sqlx::query(
            r#"
            INSERT INTO mfa_backup_codes (user_id, code_hash)
            VALUES ($1, $2)
            ON CONFLICT (user_id, code_hash) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(hash)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}

/// Every unused backup-code hash for this user, newest first.
///
/// Argon2 hashes are salted, so a code cannot be looked up by hash — the
/// service has to verify the candidate against each stored hash in turn.
pub async fn unused_backup_code_hashes(
    pool: &sqlx::PgPool,
    user_id: i64,
) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, code_hash
        FROM mfa_backup_codes
        WHERE user_id = $1
          AND used_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Burn one backup code. Returns `false` if it was already spent, which makes
/// the operation safe to race: only the first caller gets `true`.
pub async fn mark_backup_code_used(
    pool: &sqlx::PgPool,
    code_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE mfa_backup_codes
        SET used_at = now()
        WHERE id = $1
          AND used_at IS NULL
        "#,
    )
    .bind(code_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// How many codes the user still has. Surfaced in the console so nobody
/// discovers they are out of codes at the moment they need one.
pub async fn count_unused_backup_codes(
    pool: &sqlx::PgPool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM mfa_backup_codes
        WHERE user_id = $1
          AND used_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

/// Drop every backup code for a user — called when MFA is disabled, so a
/// stale printout can never be used against a re-enrolled account.
pub async fn delete_backup_codes(pool: &sqlx::PgPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM mfa_backup_codes WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
