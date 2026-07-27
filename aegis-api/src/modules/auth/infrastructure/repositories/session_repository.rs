use crate::modules::auth::domain::session::new_session::NewSession;
use crate::modules::auth::domain::session::session::Session;
use crate::modules::auth::domain::session::session_status::SessionStatus;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// INSERT
// ---------------------------------------------------------------------------

pub async fn insert_session(pool: &PgPool, session: NewSession) -> Result<Session, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (
            family_id,
            user_id,
            refresh_token_hash,
            jti,
            device_name,
            ip_address,
            user_agent,
            status,
            rotated_from,
            created_at,
            expires_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            'active',
            $8,
            now(),
            now() + interval '7 days'
        )
        RETURNING *
        "#,
    )
    .bind(session.family_id)
    .bind(session.user_id)
    .bind(session.refresh_token_hash)
    .bind(session.jti)
    .bind(session.device_name)
    .bind(session.ip_address)
    .bind(session.user_agent)
    .bind(session.rotated_from)
    .fetch_one(pool)
    .await
}

// ---------------------------------------------------------------------------
// FIND
// ---------------------------------------------------------------------------

pub async fn find_valid_session_by_jti(
    pool: &PgPool,
    jti: Uuid,
) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT *
        FROM sessions
        WHERE jti = $1
          AND status = 'active'
          AND expires_at > now()
        "#,
    )
    .bind(jti)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_jti_raw(pool: &PgPool, jti: Uuid) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT *
        FROM sessions
        WHERE jti = $1
        "#,
    )
    .bind(jti)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT *
        FROM sessions
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ---------------------------------------------------------------------------
// REVOKE
// ---------------------------------------------------------------------------

pub async fn revoke_session(pool: &PgPool, jti: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'revoked',
            last_used_at = now()
        WHERE jti = $1
          AND status = 'active'
        "#,
    )
    .bind(jti)
    .execute(pool)
    .await?;

    Ok(res.rows_affected() > 0)
}

pub async fn revoke_all_user_sessions(pool: &PgPool, user_id: i64) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'revoked',
            last_used_at = now()
        WHERE user_id = $1
          AND status = 'active'
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn revoke_family(pool: &PgPool, family_id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'revoked',
            last_used_at = now()
        WHERE family_id = $1
          AND status = 'active'
        "#,
    )
    .bind(family_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

// ---------------------------------------------------------------------------
// ROTATE
// ---------------------------------------------------------------------------

pub async fn rotate_session_atomic(
    pool: &PgPool,
    session_id: Uuid,
    new_jti: Uuid,
    new_hash: String,
) -> Result<Option<Session>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let old_session = sqlx::query_as::<_, Session>(
        r#"
        SELECT *
        FROM sessions
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(session_id)
    .fetch_optional(&mut *tx)
    .await?;

    let old_session = match old_session {
        Some(session) => session,
        None => {
            tx.rollback().await?;
            return Ok(None);
        }
    };

    if old_session.status != SessionStatus::Active || old_session.expires_at <= chrono::Utc::now() {
        tx.rollback().await?;
        return Ok(None);
    }

    sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'rotated',
            last_used_at = now()
        WHERE id = $1
          AND status = 'active'
        "#,
    )
    .bind(old_session.id)
    .execute(&mut *tx)
    .await?;

    let new_session = sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (
            family_id,
            user_id,
            refresh_token_hash,
            jti,
            device_name,
            ip_address,
            user_agent,
            status,
            rotated_from,
            created_at,
            expires_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            'active',
            $8,
            now(),
            now() + interval '7 days'
        )
        RETURNING *
        "#,
    )
    .bind(old_session.family_id)
    .bind(old_session.user_id)
    .bind(new_hash)
    .bind(new_jti)
    .bind(old_session.device_name)
    .bind(old_session.ip_address)
    .bind(old_session.user_agent)
    .bind(old_session.id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Some(new_session))
}

// ---------------------------------------------------------------------------
// TOUCH
// ---------------------------------------------------------------------------

pub async fn touch_session(pool: &PgPool, jti: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET last_used_at = now()
        WHERE jti = $1
          AND status = 'active'
        "#,
    )
    .bind(jti)
    .execute(pool)
    .await?;

    Ok(())
}
