use crate::common as helpers;
use crate::common::app::spawn_test_app;
use axum::http::StatusCode;
use helpers::{
    auth::{login_user, register_user},
    fixtures::{PASSWORD, unique_email},
};

#[tokio::test]
async fn register_user_successfully() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("register");

    let status = register_user(&app, &email, PASSWORD).await?;

    assert_eq!(status, StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn login_with_valid_credentials_returns_success() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("login");

    register_user(&app, &email, PASSWORD).await?;

    let (status, body) = login_user(&app, &email, PASSWORD).await?;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["access_token"].is_string());
    assert!(body["data"]["refresh_token"].is_string());
    assert!(body["data"]["jti"].is_string());

    Ok(())
}

#[tokio::test]
async fn login_with_wrong_password_returns_unauthorized() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("wrong-password");

    register_user(&app, &email, PASSWORD).await?;

    let (status, body) = login_user(&app, &email, "WrongPassword123!").await?;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "AUTH_UNAUTHORIZED");

    Ok(())
}
