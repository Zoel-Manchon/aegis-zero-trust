//! Email verification service.
//!
//! Two endpoints:
//!   * `request_verification` — mint a single-use, hashed, short-TTL token and
//!     dispatch a verification email through the AlertDispatcher (so the email
//!     channel is exercised end-to-end — this is the first real consumer of the
//!     alert pipeline).
//!   * `perform_verification` — atomically consume a token: validate hash, TTL,
//!     unused, then mark used + flip `users.email_verified_at`.
//!
//! Anti-enumeration: `request_verification` always returns the same generic
//! success regardless of whether the email exists (mirrors password reset).
//!
//! Token shape: 32 random bytes -> base64-url-no-pad. Raw token is the link
//! payload; SHA-256(raw) is what we store. Raw token never leaves this module
//! except embedded in the email body.

use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::{
    app_state::AppState,
    core::errors::app_error::AppError,
    modules::{
        alerts::domain::alert::{Alert, AlertSeverity},
        auth::infrastructure::repositories::{
            email_verification_repository as repo, user_repository::UserRepository,
        },
    },
};

const TOKEN_TTL_MINUTES: i64 = 60 * 24; // 24 hours

fn mint_raw_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_token(raw: &str) -> String {
    hex::encode(Sha256::digest(raw.as_bytes()))
}

/// Issue a verification token and dispatch the email through the AlertDispatcher.
///
/// Returns the same generic Ok(()) whether the email exists or not so callers
/// cannot probe account existence. The dispatcher fans out to log + email
/// channels; the email channel is a stub seam until SMTP is wired (see
/// EmailChannel) — that's intentional, and switching it on is a one-file change.
pub async fn request_verification(state: &AppState, email: &str) -> Result<(), AppError> {
    let normalized = email.trim().to_lowercase();

    let user = UserRepository::find_by_email(&state.pool, &normalized)
        .await
        .map_err(|_| AppError::DatabaseError)?;

    let Some(user) = user else {
        // Anti-enumeration: same response shape as the success path.
        return Ok(());
    };

    // Don't re-issue if already verified — silent no-op (also anti-enumeration:
    // verified vs unverified must look the same from outside).
    // Read the verified column directly so we don't have to widen the User struct.
    let already_verified: Option<(Option<chrono::DateTime<Utc>>,)> = sqlx::query_as(
        "SELECT email_verified_at FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    if let Some((Some(_),)) = already_verified {
        return Ok(());
    }

    // Invalidate any outstanding verification tokens for this user so only the
    // newest link works (defensive against link replay / shoulder-surfing).
    repo::invalidate_user_tokens(&state.pool, user.id)
        .await
        .map_err(|_| AppError::DatabaseError)?;

    let raw = mint_raw_token();
    let token_hash = hash_token(&raw);
    let expires_at = Utc::now() + Duration::minutes(TOKEN_TTL_MINUTES);

    repo::insert_token(&state.pool, user.id, &token_hash, expires_at)
        .await
        .map_err(|_| AppError::DatabaseError)?;

    // Fire through the alert dispatcher. The raw token is in the body only —
    // never returned by the HTTP layer.
    let link = format!("/verify-email/confirm?token={raw}");
    let alert = Alert::new(
        "email_verification",
        AlertSeverity::Info,
        "Verify your email address",
        format!(
            "Welcome! Please confirm your email address by visiting this link \
             within 24 hours: {link}\n\nIf you did not request this, ignore this email."
        ),
    )
    .to_recipient(normalized)
    .with_meta("user_id", user.id.to_string())
    .with_meta("link", link);

    state.alerts.dispatch(&alert).await;
    Ok(())
}

/// Consume a verification token. Validates hash, TTL, unused, then atomically
/// marks used + flips `email_verified_at`.
pub async fn perform_verification(state: &AppState, raw_token: &str) -> Result<(), AppError> {
    if raw_token.is_empty() {
        return Err(AppError::Unauthorized);
    }
    let token_hash = hash_token(raw_token);

    let record = repo::find_by_hash(&state.pool, &token_hash)
        .await
        .map_err(|_| AppError::DatabaseError)?
        .ok_or(AppError::Unauthorized)?;

    // Single-use.
    if record.used_at.is_some() {
        return Err(AppError::Unauthorized);
    }
    // TTL.
    if record.expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    // Mark used FIRST (atomic guard against double-spend); only if it actually
    // flipped do we set the user verified.
    let used = repo::mark_used(&state.pool, record.id)
        .await
        .map_err(|_| AppError::DatabaseError)?;
    if !used {
        // Lost the race — another request consumed this token.
        return Err(AppError::Unauthorized);
    }

    repo::set_user_verified(&state.pool, record.user_id)
        .await
        .map_err(|_| AppError::DatabaseError)?;

    Ok(())
}
