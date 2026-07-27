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