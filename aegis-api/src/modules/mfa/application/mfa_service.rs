// =============================================================================
// MFA service — TOTP enrollment, verification, and recovery.
//
// HARDENING STATUS
//   [x] backup/recovery codes, Argon2id-hashed at rest, single use
//   [x] last-used timestep stored, so a code cannot be replayed in its window
//   [ ] encrypted-at-rest TOTP secrets (Vault/KMS envelope encryption)
//   [ ] per-user MFA attempt throttling beyond the per-IP limiter
//   [x] WebAuthn/passkeys for phishing-resistant MFA (see modules::passkeys)
//
// The two open items are tracked in the README's security section. Both are
// deliberate: they need infrastructure decisions, not just code.
// =============================================================================

use crate::{
    core::{errors::app_error::AppError, security::hash_password::{hash_password, verify_password}},
    modules::mfa::infrastructure::repositories::mfa_repository,
};

use data_encoding::BASE32_NOPAD;
use rand::{RngCore, rngs::OsRng};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

/// TOTP parameters. Changing STEP_SECONDS invalidates every enrolled device.
const STEP_SECONDS: u64 = 30;
const SKEW_STEPS: i64 = 1;
const BACKUP_CODE_COUNT: usize = 10;

pub struct MfaSetup {
    pub secret: String,
    pub otpauth_url: String,
}

/// Which factor satisfied the challenge. The caller audits these differently:
/// a backup code is a legitimate but noteworthy event — it means the user lost
/// their authenticator, and it is also what an attacker would reach for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondFactor {
    Totp,
    BackupCode,
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

/// Confirm enrollment and mint the first set of backup codes.
///
/// The codes are returned in plaintext exactly once, here. Only their Argon2
/// hashes are stored, so this response is the user's single chance to save
/// them — the same contract every serious IdP uses.
pub async fn verify_setup(
    pool: &sqlx::PgPool,
    user_id: i64,
    code: &str,
) -> Result<Option<Vec<String>>, AppError> {
    let mfa = mfa_repository::find_by_user_id(pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if mfa.enabled {
        return Err(AppError::Conflict);
    }

    let Some(step) = matching_step(&mfa.secret, code) else {
        return Ok(None);
    };

    mfa_repository::enable_mfa(pool, user_id).await?;
    // Burn the enrollment step immediately: the code just shown on screen must
    // not also work as the first login code.
    mfa_repository::try_consume_step(pool, user_id, step).await?;

    let codes = regenerate_backup_codes(pool, user_id).await?;

    Ok(Some(codes))
}

/// Verify a TOTP code, rejecting replays.
///
/// `check_current` only answers "is this code valid right now", which is not
/// enough: the same code stays valid for the rest of its step. We resolve the
/// exact step the code belongs to and claim it atomically, so a code works
/// once and only once.
pub async fn verify_code(pool: &sqlx::PgPool, user_id: i64, code: &str) -> Result<bool, AppError> {
    let mfa = mfa_repository::find_by_user_id(pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !mfa.enabled {
        return Err(AppError::Unauthorized);
    }

    let Some(step) = matching_step(&mfa.secret, code) else {
        return Ok(false);
    };

    // A valid code whose step is already spent is a replay, not a success.
    mfa_repository::try_consume_step(pool, user_id, step)
        .await
        .map_err(AppError::from)
}

/// Verify either factor: TOTP first, then an unused backup code.
///
/// Backup codes are longer than 6 digits, so the two never collide, and trying
/// TOTP first keeps the common path cheap — Argon2 verification is deliberately
/// slow and we only pay for it on actual recovery.
pub async fn verify_second_factor(
    pool: &sqlx::PgPool,
    user_id: i64,
    code: &str,
) -> Result<Option<SecondFactor>, AppError> {
    if code.len() == 6 && verify_code(pool, user_id, code).await? {
        return Ok(Some(SecondFactor::Totp));
    }

    if consume_backup_code(pool, user_id, code).await? {
        return Ok(Some(SecondFactor::BackupCode));
    }

    Ok(None)
}

/// Spend one backup code. Returns false for an unknown, already-used, or
/// malformed code — the caller must not distinguish between those cases.
pub async fn consume_backup_code(
    pool: &sqlx::PgPool,
    user_id: i64,
    code: &str,
) -> Result<bool, AppError> {
    let candidate = normalize_backup_code(code);
    if candidate.is_empty() {
        return Ok(false);
    }

    let stored = mfa_repository::unused_backup_code_hashes(pool, user_id).await?;

    for (id, hash) in stored {
        if verify_password(&candidate, &hash).unwrap_or(false) {
            // mark_backup_code_used is conditional on used_at IS NULL, so two
            // concurrent requests with the same code cannot both succeed.
            return Ok(mfa_repository::mark_backup_code_used(pool, id).await?);
        }
    }

    Ok(false)
}

/// Mint a fresh set of codes, invalidating any previous set.
pub async fn regenerate_backup_codes(
    pool: &sqlx::PgPool,
    user_id: i64,
) -> Result<Vec<String>, AppError> {
    let codes: Vec<String> = (0..BACKUP_CODE_COUNT).map(|_| generate_backup_code()).collect();

    let mut hashes = Vec::with_capacity(codes.len());
    for code in &codes {
        hashes.push(hash_password(code).map_err(|_| AppError::InternalError)?);
    }

    mfa_repository::replace_backup_codes(pool, user_id, &hashes).await?;

    Ok(codes)
}

pub async fn remaining_backup_codes(pool: &sqlx::PgPool, user_id: i64) -> Result<i64, AppError> {
    Ok(mfa_repository::count_unused_backup_codes(pool, user_id).await?)
}

/// Disabling MFA also destroys the backup codes: a printout from a previous
/// enrollment must never unlock a re-enrolled account.
pub async fn disable_mfa(pool: &sqlx::PgPool, user_id: i64, code: &str) -> Result<bool, AppError> {
    if verify_second_factor(pool, user_id, code).await?.is_none() {
        return Ok(false);
    }

    mfa_repository::disable_mfa(pool, user_id).await?;
    mfa_repository::delete_backup_codes(pool, user_id).await?;

    Ok(true)
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Resolve which TOTP step a code belongs to, scanning the accepted skew.
///
/// Returning the step (rather than a bool) is what makes replay prevention
/// possible: we need to know *which* code was spent, not merely that one was.
///
/// Note the zero-skew instance. `TOTP::check` applies the struct's own skew
/// internally, so probing a skewed TOTP at each candidate step would match on
/// the very first probe every time and report the wrong step. Comparing
/// against a skew-0 clone is what makes the answer the *actual* step.
fn matching_step(secret: &str, code: &str) -> Option<i64> {
    if code.len() != 6 || !code.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let exact = build_totp_with_skew(secret, 0).ok()?;
    let current = (now_secs() / STEP_SECONDS) as i64;

    for offset in -SKEW_STEPS..=SKEW_STEPS {
        let step = current + offset;
        if step < 0 {
            continue;
        }
        if exact.check(code, (step as u64) * STEP_SECONDS) {
            return Some(step);
        }
    }

    None
}

fn generate_base32_secret() -> String {
    let mut bytes = [0u8; 20];
    OsRng.fill_bytes(&mut bytes);
    BASE32_NOPAD.encode(&bytes)
}

/// A backup code: 10 Crockford-ish base32 characters, grouped as XXXXX-XXXXX.
/// ~50 bits of entropy, which is far beyond brute-forcing an Argon2 hash, and
/// still short enough to write on paper without mistakes.
fn generate_backup_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTVWXYZ23456789"; // no I, L, O, 0, 1, U
    let mut bytes = [0u8; 10];
    OsRng.fill_bytes(&mut bytes);

    let chars: String = bytes
        .iter()
        .map(|b| ALPHABET[*b as usize % ALPHABET.len()] as char)
        .collect();

    format!("{}-{}", &chars[..5], &chars[5..])
}

/// Accept a code however the user typed it: with or without the dash, in any
/// case, with stray whitespace. Anything outside the alphabet is dropped.
fn normalize_backup_code(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if cleaned.len() != 10 {
        return String::new();
    }

    format!("{}-{}", &cleaned[..5], &cleaned[5..])
}

fn build_totp(secret: &str, account_name: &str) -> Result<TOTP, AppError> {
    build_totp_named(secret, account_name, SKEW_STEPS as u8)
}

/// Skew-controlled builder. Only `matching_step` needs skew 0; everything that
/// shows a QR uses the normal instance so the otpauth URL stays correct.
fn build_totp_with_skew(secret: &str, skew: u8) -> Result<TOTP, AppError> {
    build_totp_named(secret, "user", skew)
}

fn build_totp_named(secret: &str, account_name: &str, skew: u8) -> Result<TOTP, AppError> {
    let secret = Secret::Encoded(secret.to_string())
        .to_bytes()
        .map_err(|_| AppError::InternalError)?;

    TOTP::new(
        Algorithm::SHA1,
        6,
        skew,
        STEP_SECONDS,
        secret,
        Some("Aegis".to_string()),
        account_name.to_string(),
    )
    .map_err(|_| AppError::InternalError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_codes_are_grouped_and_unambiguous() {
        let code = generate_backup_code();
        assert_eq!(code.len(), 11);
        assert_eq!(code.as_bytes()[5], b'-');
        assert!(!code.contains('I') && !code.contains('O') && !code.contains('0'));
    }

    #[test]
    fn normalize_accepts_any_reasonable_transcription() {
        let code = generate_backup_code();
        let squashed = code.replace('-', "").to_lowercase();
        assert_eq!(normalize_backup_code(&squashed), code);
        assert_eq!(normalize_backup_code(&format!("  {code} ")), code);
    }

    #[test]
    fn normalize_rejects_wrong_length() {
        assert_eq!(normalize_backup_code("ABC"), "");
        assert_eq!(normalize_backup_code("123456"), "");
    }

    #[test]
    fn generated_codes_do_not_repeat() {
        let a = generate_backup_code();
        let b = generate_backup_code();
        assert_ne!(a, b);
    }
}
