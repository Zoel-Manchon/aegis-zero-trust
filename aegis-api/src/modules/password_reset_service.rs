//! Password reset (forgot/reset) application logic.
//!
//! This is the security-critical core. Design decisions, all deliberate:
//!
//! 1. NO USER ENUMERATION. `request_reset` returns the same `Ok(())` whether or
//!    not the email exists. The handler returns an identical response either
//!    way, so an attacker cannot probe which emails are registered.
//!
//! 2. TOKEN IS A SECRET, STORED HASHED. We generate 32 bytes of CSPRNG
//!    randomness, hex-encode it as the raw token (given to the user), and store
//!    only its SHA-256 hash. A database leak yields hashes, not usable links.
//!
//! 3. SINGLE-USE + SHORT TTL. Tokens expire after `TOKEN_TTL_MINUTES` and are
//!    marked used on first successful reset. Requesting a new token invalidates
//!    prior ones, so at most one link is live.
//!
//! 4. SESSION INVALIDATION ON RESET. After a successful password change we
//!    revoke ALL of the user's sessions. If the account was compromised, the
//!    attacker's refresh tokens die with the reset.
//!
//! 5. CONSTANT-TIME-ISH LOOKUP. We hash the incoming token and look it up by
//!    hash; we never compare raw tokens. (The hash lookup is in the DB; we do
//!    not branch on raw token contents.)
//!
//! Delivery of the link is abstracted: for now we LOG it. When the alerts/email
//! channel exists, swap the `deliver_reset_link` body to send a real email. The
//! raw token is NEVER returned in the HTTP response.

use crate::core::errors::app_error::AppError;
use crate::core::security::hash_password::hash_password;
use crate::modules::auth::infrastructure::repositories::{
    password_reset_repository as reset_repo, session_repository, user_repository::UserRepository,
};
use chrono::{Duration, Utc};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

/// How long a reset token is valid.
const TOKEN_TTL_MINUTES: i64 = 30;
/// Minimum new-password length (matches register policy).
const MIN_PASSWORD_LEN: usize = 12;

/// Hash a raw token for storage/lookup. SHA-256 is appropriate here because the
/// token is already high-entropy (256 bits); we are not stretching a low-entropy
/// secret, just avoiding storing the raw value.
fn hash_token(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    hex::encode(digest)
}

/// Generate a 256-bit CSPRNG token, hex-encoded (64 chars).
fn generate_raw_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Step 1: a user asks to reset their password.
///
/// ALWAYS returns `Ok(())` regardless of whether the email exists, so the caller
/// can return an identical response and prevent enumeration. If the email does
/// exist, we mint a fresh token (invalidating any prior ones) and "deliver" the
/// link.
pub async fn request_reset(pool: &sqlx::PgPool, email: &str) -> Result<(), AppError> {
    let normalized = email.trim().to_lowercase();

    // Unknown email -> do nothing, but return Ok so the response is identical.
    let user = match UserRepository::find_by_email(pool, &normalized).await {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(()),
        Err(_) => return Err(AppError::DatabaseError),
    };

    // Only one live token at a time.
    let _ = reset_repo::invalidate_user_tokens(pool, user.id).await;

    let raw_token = generate_raw_token();
    let token_hash = hash_token(&raw_token);
    let expires_at = Utc::now() + Duration::minutes(TOKEN_TTL_MINUTES);

    reset_repo::insert_token(pool, user.id, &token_hash, expires_at)
        .await
        .map_err(|_| AppError::DatabaseError)?;

    deliver_reset_link(&normalized, &raw_token).await;

    Ok(())
}

/// Step 2: complete the reset with the raw token and a new password.
///
/// Returns `Ok(())` on success. On any failure (bad/expired/used token, weak
/// password) returns an error; the handler maps these to generic responses.
pub async fn perform_reset(
    pool: &sqlx::PgPool,
    raw_token: &str,
    new_password: &str,
) -> Result<(), AppError> {
    if new_password.trim().is_empty() || new_password.len() < MIN_PASSWORD_LEN {
        return Err(AppError::BadRequest);
    }
    if new_password.len() > 1024 {
        return Err(AppError::BadRequest);
    }

    let token_hash = hash_token(raw_token);

    let token = reset_repo::find_by_hash(pool, &token_hash)
        .await
        .map_err(|_| AppError::DatabaseError)?
        .ok_or(AppError::Unauthorized)?;

    // Reject if already used or expired.
    if token.used_at.is_some() || token.expires_at <= Utc::now() {
        return Err(AppError::Unauthorized);
    }

    // Atomically claim the token (single-use). If this returns false, another
    // request used it first — treat as unauthorized.
    let claimed = reset_repo::mark_used(pool, token.id)
        .await
        .map_err(|_| AppError::DatabaseError)?;
    if !claimed {
        return Err(AppError::Unauthorized);
    }

    // Set the new password hash.
    let hashed = hash_password(new_password).map_err(|_| AppError::HashError)?;
    UserRepository::update_password(pool, token.user_id, &hashed)
        .await
        .map_err(|_| AppError::DatabaseError)?;

    // Zero-trust: kill every existing session for this user. A compromised
    // account cannot survive a password reset.
    let _ = session_repository::revoke_all_user_sessions(pool, token.user_id).await;

    // Belt and braces: invalidate any other outstanding tokens too.
    let _ = reset_repo::invalidate_user_tokens(pool, token.user_id).await;

    Ok(())
}

/// Deliver the reset link to the user.
///
/// PLACEHOLDER: logs the link. Swap this for the real email/alerts channel when
/// it exists. The raw token must NEVER be returned to the HTTP caller — only
/// delivered out-of-band to the verified email address.
async fn deliver_reset_link(email: &str, raw_token: &str) {
    // In production this is a templated email to `email`. For now, log it so a
    // developer can complete the flow locally. Do not log in production builds.
    tracing::info!(
        target: "password_reset",
        email = %email,
        reset_link = %format!("/password/reset?token={raw_token}"),
        "password reset link generated (DEV: delivered via log)"
    );
}
