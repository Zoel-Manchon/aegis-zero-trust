
// HARDENING NOTE:
// TOTP is good as a baseline second factor, but this service still needs:
// - encrypted-at-rest secrets (Vault/KMS envelope encryption)
// - last-used timestep storage to prevent code replay within the same window
// - per-user/per-token MFA attempt throttling
// - backup/recovery codes hashed at rest
// - WebAuthn/passkeys for phishing-resistant MFA
use crate::{
    core::errors::app_error::AppError, modules::mfa::infrastructure::repositories::mfa_repository,
};

use data_encoding::BASE32_NOPAD;
use rand::{RngCore, rngs::OsRng};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct MfaSetup {
    pub secret: String,
    pub otpauth_url: String,
}

pub async fn setup_mfa(
    pool: &sqlx::PgPool,
    user_id: i64,
    user_email: &str,
) -> Result<MfaSetup, AppError> {
    if let Some(existing) = mfa_repository::find_by_user_id(pool, user_id).await? {
        if existing.enabled {
            return Err(AppError::Conflict);
        }

        let otpauth_url = build_totp(&existing.secret, user_email)?.get_url();

        return Ok(MfaSetup {
            secret: existing.secret,
            otpauth_url,
        });
    }

    let secret = generate_base32_secret();

    let row = mfa_repository::create_mfa_secret(pool, user_id, &secret).await?;

    let otpauth_url = build_totp(&row.secret, user_email)?.get_url();

    Ok(MfaSetup {
        secret: row.secret,
        otpauth_url,
    })
}

pub async fn verify_setup(pool: &sqlx::PgPool, user_id: i64, code: &str) -> Result<bool, AppError> {
    let mfa = mfa_repository::find_by_user_id(pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if mfa.enabled {
        return Ok(true);
    }

    let totp = build_totp(&mfa.secret, "user")?;

    if !totp
        .check_current(code)
        .map_err(|_| AppError::Unauthorized)?
    {
        return Ok(false);
    }

    mfa_repository::enable_mfa(pool, user_id).await?;

    Ok(true)
}

pub async fn verify_code(pool: &sqlx::PgPool, user_id: i64, code: &str) -> Result<bool, AppError> {
    let mfa = mfa_repository::find_by_user_id(pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !mfa.enabled {
        return Err(AppError::Unauthorized);
    }

    let totp = build_totp(&mfa.secret, "user")?;

    let valid = totp
        .check_current(code)
        .map_err(|_| AppError::Unauthorized)?;

    Ok(valid)
}

pub async fn disable_mfa(pool: &sqlx::PgPool, user_id: i64, code: &str) -> Result<bool, AppError> {
    let valid = verify_code(pool, user_id, code).await?;

    if !valid {
        return Ok(false);
    }

    mfa_repository::disable_mfa(pool, user_id).await?;

    Ok(true)
}

fn generate_base32_secret() -> String {
    let mut bytes = [0u8; 20];
    OsRng.fill_bytes(&mut bytes);
    BASE32_NOPAD.encode(&bytes)
}

fn build_totp(secret: &str, account_name: &str) -> Result<TOTP, AppError> {
    let secret = Secret::Encoded(secret.to_string())
        .to_bytes()
        .map_err(|_| AppError::InternalError)?;

    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("Axum Auth Playground".to_string()),
        account_name.to_string(),
    )
    .map_err(|_| AppError::InternalError)
}
