use crate::common as helpers;
use crate::common::app::spawn_test_app;
use crate::common::fixtures::unique_email;
use crate::common::mfa::enable_mfa_for;
use axum::http::StatusCode;
use helpers::auth::{login_user, post_json};
use serde_json::json;

#[tokio::test]
async fn invalid_mfa_code_is_rejected() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("mfa-flow");
    let user = enable_mfa_for(&app, &email).await?;

    let (login_status, login_body) = login_user(&app, &user.email, &user.password).await?;
    assert_eq!(login_status, StatusCode::OK);
    let mfa_token = login_body["data"]["mfa_token"]
        .as_str()
        .expect("mfa token missing");

    let (status, body) = post_json(
        &app,
        "/mfa/complete-login",
        json!({ "mfa_token": mfa_token, "code": "000000" }),
    )
    .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "AUTH_UNAUTHORIZED");

    Ok(())
}
/// The per-IP limiter would let a botnet walk the six-digit space from rotating
/// addresses, so the account itself carries a failure budget. Six wrong codes
/// against one account must lock it regardless of where they came from.
#[tokio::test]
async fn repeated_wrong_codes_lock_the_account() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("mfa-throttle");
    let user = enable_mfa_for(&app, &email).await?;

    let (login_status, login_body) = login_user(&app, &user.email, &user.password).await?;
    assert_eq!(login_status, StatusCode::OK);
    let mfa_token = login_body["data"]["mfa_token"]
        .as_str()
        .expect("mfa token missing")
        .to_string();

    let mut last_status = StatusCode::OK;
    for attempt in 0..6 {
        let (status, _body) = post_json(
            &app,
            "/mfa/complete-login",
            json!({ "mfa_token": mfa_token, "code": format!("00000{attempt}") }),
        )
        .await?;
        last_status = status;
    }

    assert_eq!(
        last_status,
        StatusCode::TOO_MANY_REQUESTS,
        "the account should be locked once its failure budget is spent"
    );

    // And the lock outlives a correct code: the legitimate owner cannot unlock
    // early, which is what stops an attacker from racing the real user's login.
    let (status, _body) = post_json(
        &app,
        "/mfa/complete-login",
        json!({ "mfa_token": mfa_token, "code": user.current_code() }),
    )
    .await?;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);

    Ok(())
}
