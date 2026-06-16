use crate::common as helpers;

use crate::common::app::spawn_test_app;
use axum::http::StatusCode;
use helpers::{
    auth::{get_with_bearer, login_user, register_user},
    fixtures::{PASSWORD, unique_email},
};

#[tokio::test]
async fn normal_user_cannot_access_admin_dashboard() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("rbac-user");

    register_user(&app, &email, PASSWORD).await?;

    let (login_status, login_body) = login_user(&app, &email, PASSWORD).await?;
    assert_eq!(login_status, StatusCode::OK);

    let access_token = login_body["data"]["access_token"]
        .as_str()
        .expect("access token should exist");

    let (status, body) = get_with_bearer(&app, "/admin/dashboard", access_token).await?;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "AUTH_UNAUTHORIZED");

    Ok(())
}
