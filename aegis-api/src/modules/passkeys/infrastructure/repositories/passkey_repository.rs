use crate::core::errors::app_error::AppError;
use crate::modules::passkeys::domain::passkey_credential::PasskeyCredential;
use sqlx::PgPool;

/// Repository for WebAuthn credentials.
///
/// HARDENING NOTE:
/// Use constant-time credential-id comparison where possible and never expose
/// whether a credential exists to unauthenticated callers. Authentication flows
/// should return a generic unauthorized error.
pub async fn list_user_passkeys(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<PasskeyCredential>, AppError> {
    let rows = sqlx::query_as!(
        PasskeyCredential,
        r#"
        SELECT id, user_id, credential_id, public_key_cose, sign_count,
               friendly_name, transports, aaguid, created_at, last_used_at, revoked_at
        FROM passkey_credentials
        WHERE user_id = $1 AND revoked_at IS NULL
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_active_by_credential_id(
    pool: &PgPool,
    credential_id: &str,
) -> Result<Option<PasskeyCredential>, AppError> {
    let row = sqlx::query_as!(
        PasskeyCredential,
        r#"
        SELECT id, user_id, credential_id, public_key_cose, sign_count,
               friendly_name, transports, aaguid, created_at, last_used_at, revoked_at
        FROM passkey_credentials
        WHERE credential_id = $1 AND revoked_at IS NULL
        "#,
        credential_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn insert_passkey(
    pool: &PgPool,
    user_id: i64,
    credential_id: &str,
    public_key_cose: &[u8],
    sign_count: i64,
    friendly_name: Option<&str>,
    transports: &[String],
    aaguid: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO passkey_credentials (
            user_id, credential_id, public_key_cose, sign_count,
            friendly_name, transports, aaguid
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_id,
        credential_id,
        public_key_cose,
        sign_count,
        friendly_name,
        transports,
        aaguid
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_successful_assertion(
    pool: &PgPool,
    credential_id: &str,
    new_sign_count: i64,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE passkey_credentials
        SET sign_count = $2, last_used_at = now()
        WHERE credential_id = $1 AND revoked_at IS NULL
        "#,
        credential_id,
        new_sign_count
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_passkey(
    pool: &PgPool,
    user_id: i64,
    credential_id: &str,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE passkey_credentials
        SET revoked_at = now()
        WHERE user_id = $1 AND credential_id = $2 AND revoked_at IS NULL
        "#,
        user_id,
        credential_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
