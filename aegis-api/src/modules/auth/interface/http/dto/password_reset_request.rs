//! Request DTOs for the password-reset endpoints.
//!
//! Mirrors the validation style of the existing login/register DTOs (uses the
//! `validator` crate). Field names map directly to the JSON body.

use serde::Deserialize;
use validator::Validate;

/// Body for `POST /password/forgot`.
#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

/// Body for `POST /password/reset`.
#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    /// The raw token from the reset link.
    #[validate(length(min = 16, max = 256))]
    pub token: String,

    /// The new password. Full strength policy is enforced in the service; this
    /// is a cheap first-pass bound.
    #[validate(length(min = 12, max = 1024))]
    pub new_password: String,
}
