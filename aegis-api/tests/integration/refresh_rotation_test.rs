use crate::common as helpers;

use crate::common::app::spawn_test_app;
use axum::http::StatusCode;
use helpers::{
    auth::{login_user, post_with_bearer, register_user},
    fixtures::{PASSWORD, unique_email},
};
use serde_json::json;

#[tokio::test]
async fn refresh_token_rotation_blocks_replay() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("refresh");

    register_user(&app, &email, PASSWORD).await?;

    let (login_status, login_body) = login_user(&app, &email, PASSWORD).await?;

    assert_eq!(login_status, StatusCode::OK);

    let refresh_token = login_body["data"]["refresh_token"]
        .as_str()
        .expect("refresh token missing");

    let jti = login_body["data"]["jti"].as_str().expect("jti missing");

    // -------------------------------------------------
    // First refresh should succeed
    // -------------------------------------------------

    let (first_status, _) = post_with_bearer(
        &app,
        "/refresh",
        "",
        json!({
            "refresh_token": refresh_token,
            "jti": jti
        }),
    )
    .await?;

    assert_eq!(first_status, StatusCode::OK);

    // -------------------------------------------------
    // Replay old refresh token should fail
    // -------------------------------------------------

    let (replay_status, replay_body) = post_with_bearer(
        &app,
        "/refresh",
        "",
        json!({
            "refresh_token": refresh_token,
            "jti": jti
        }),
    )
    .await?;

    assert_eq!(replay_status, StatusCode::UNAUTHORIZED);

    assert_eq!(replay_body["error"]["code"], "AUTH_UNAUTHORIZED");

    Ok(())
}
