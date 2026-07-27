//! MFA test helpers.
//!
//! Drive the real MFA endpoints to put a freshly-registered user into the
//! "MFA enabled" state, then generate valid TOTP codes on demand. Replaces the
//! old assumption that a pre-seeded `test@example.com` existed.
//!
//! TOTP params MUST match the server's `build_totp`:
//!   Algorithm::SHA1, 6 digits, skew 1, step 30s, Secret::Encoded (base32).

use crate::common::app::TestApp;
use crate::common::auth::{login_user, post_with_bearer, register_user};
use crate::common::fixtures::PASSWORD;
use totp_rs::{Algorithm, Secret, TOTP};

pub struct MfaUser {
    pub email: String,
    pub password: String,
    pub secret: String,
}

impl MfaUser {
    pub fn current_code(&self) -> String {
        build_test_totp(&self.secret)
            .generate_current()
            .expect("generate TOTP")
    }
}

fn build_test_totp(secret: &str) -> TOTP {
    let bytes = Secret::Encoded(secret.to_string())
        .to_bytes()
        .expect("decode base32 secret");
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        bytes,
        Some("test-issuer".to_string()),
        "test-account".to_string(),
    )
    .expect("build test TOTP")
}

pub async fn enable_mfa_for(app: &TestApp, email: &str) -> anyhow::Result<MfaUser> {
    let status = register_user(app, email, PASSWORD).await?;
    assert!(
        status.is_success() || status.as_u16() == 409,
        "register failed for {email}: {status}"
    );

    let (login_status, login_body) = login_user(app, email, PASSWORD).await?;
    assert!(login_status.is_success(), "pre-MFA login failed: {login_status}");
    let access_token = login_body["data"]["access_token"]
        .as_str()
        .expect("access token missing from pre-MFA login")
        .to_string();

    let (setup_status, setup_body) =
        post_with_bearer(app, "/mfa/setup", &access_token, serde_json::json!({})).await?;
    assert!(setup_status.is_success(), "/mfa/setup failed: {setup_status}");
    let secret = setup_body["data"]["secret"]
        .as_str()
        .expect("secret missing from /mfa/setup response")
        .to_string();

    let code = build_test_totp(&secret)
        .generate_current()
        .expect("generate TOTP");
    let (verify_status, _) = post_with_bearer(
        app,
        "/mfa/verify-setup",
        &access_token,
        serde_json::json!({ "code": code }),
    )
    .await?;
    assert!(verify_status.is_success(), "/mfa/verify-setup failed: {verify_status}");

    Ok(MfaUser {
        email: email.to_string(),
        password: PASSWORD.to_string(),
        secret,
    })
}
