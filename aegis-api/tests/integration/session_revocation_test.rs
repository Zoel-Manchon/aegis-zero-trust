use crate::common as helpers;

use axum::http::StatusCode;

use crate::common::app::spawn_test_app;
use helpers::{
    auth::{get_with_bearer, login_user, post_with_bearer, register_user},
    fixtures::{PASSWORD, unique_email},
};
use serde_json::json;
#[tokio::test]
async fn revoked_session_cannot_access_protected_routes() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("session");

    register_user(&app, &email, PASSWORD).await?;

    let (login_status, login_body) = login_user(&app, &email, PASSWORD).await?;

    assert_eq!(login_status, StatusCode::OK);

    let access_token = login_body["data"]["access_token"]
        .as_str()
        .expect("access token missing");

    // -------------------------------------------------
    // Access before logout should work
    // -------------------------------------------------

    let (before_status, _) = get_with_bearer(&app, "/logout", access_token).await?;

    assert_ne!(before_status, StatusCode::UNAUTHORIZED);

    // -------------------------------------------------
    // Logout
    // -------------------------------------------------

    let (logout_status, _) = post_with_bearer(&app, "/logout", access_token, json!({})).await?;

    assert_eq!(logout_status, StatusCode::NO_CONTENT);

    // -------------------------------------------------
    // Token should now be dead
    // -------------------------------------------------

    let (after_status, after_body) =
        post_with_bearer(&app, "/logout", access_token, json!({})).await?;

    assert_eq!(after_status, StatusCode::UNAUTHORIZED);

    assert_eq!(after_body["error"]["code"], "AUTH_UNAUTHORIZED");

    Ok(())
}
