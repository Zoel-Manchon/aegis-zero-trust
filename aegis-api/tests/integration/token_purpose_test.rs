use crate::common::app::spawn_test_app;
use crate::common::auth::{get_with_bearer, login_user};
use crate::common::fixtures::unique_email;
use crate::common::mfa::enable_mfa_for;
use axum::http::StatusCode;

/// An MFA-purpose token (issued at the MFA challenge step of login) must NOT be
/// accepted as an access token on a protected route.
///
/// This guards against token-purpose confusion: `verify_access` in jwt.rs
/// rejects any token whose `purpose != "access"`, so the auth middleware fails
/// the request with 401 before any handler or RBAC check runs. The mfa_token is
/// only usable at `/mfa/complete-login`.
///
/// Built on a fresh user (register + enable MFA via the real endpoints) so it
/// never depends on pre-seeded database rows.
#[tokio::test]
async fn mfa_token_cannot_access_protected_route() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    // Fresh user with MFA enabled.
    let email = unique_email("token-purpose");
    let user = enable_mfa_for(&app, &email).await?;

    // With MFA enabled, login returns an MFA challenge token rather than access
    // tokens.
    let (login_status, login_body) = login_user(&app, &user.email, &user.password).await?;
    assert_eq!(login_status, StatusCode::OK);
    let mfa_token = login_body["data"]["mfa_token"]
        .as_str()
        .expect("login should return an mfa_token once MFA is enabled");

    // Use the mfa_token as a Bearer access token on a real protected GET route.
    // The auth middleware must reject it (purpose != "access") -> 401.
    let (status, body) = get_with_bearer(&app, "/admin/dashboard", mfa_token).await?;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "mfa-purpose token must be rejected as an access token, body={body:#?}"
    );
    assert_eq!(body["error"]["code"], "AUTH_UNAUTHORIZED");

    Ok(())
}
