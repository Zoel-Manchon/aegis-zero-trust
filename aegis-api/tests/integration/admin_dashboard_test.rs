use crate::common as helpers;
use crate::common::app::spawn_test_app;
use axum::http::StatusCode;
use helpers::{
    auth::{get_with_bearer, login_user, register_user},
    fixtures::{unique_email, PASSWORD},
};

/// An admin user can reach the admin dashboard and the response echoes their
/// role. Mirrors the admin-promotion pattern used by admin_security_api_test.
#[tokio::test]
async fn admin_can_access_dashboard() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("admin-dash");
    let register_status = register_user(&app, &email, PASSWORD).await?;
    assert_eq!(register_status, StatusCode::OK);

    // Promote to admin directly in the DB.
    sqlx::query("UPDATE users SET user_role = 'admin' WHERE email = $1")
        .bind(&email)
        .execute(&app.state.pool)
        .await?;

    let (login_status, login_body) = login_user(&app, &email, PASSWORD).await?;
    assert_eq!(login_status, StatusCode::OK);
    let access_token = login_body["data"]["access_token"]
        .as_str()
        .expect("access token missing");

    let (status, body) = get_with_bearer(&app, "/admin/dashboard", access_token).await?;

    assert_eq!(status, StatusCode::OK, "admin should reach dashboard, body={body:#?}");
    assert_eq!(body["data"]["role"], "admin");
    assert!(
        body["data"]["message"].is_string(),
        "dashboard should return a message, body={body:#?}"
    );

    Ok(())
}

/// A normal (non-admin) user is denied at the admin dashboard. The current RBAC
/// implementation surfaces this denial as 401 AUTH_UNAUTHORIZED (consistent with
/// rbac_policy_test).
#[tokio::test]
async fn normal_user_denied_dashboard() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("normal-dash");
    register_user(&app, &email, PASSWORD).await?;

    let (login_status, login_body) = login_user(&app, &email, PASSWORD).await?;
    assert_eq!(login_status, StatusCode::OK);
    let access_token = login_body["data"]["access_token"]
        .as_str()
        .expect("access token missing");

    let (status, body) = get_with_bearer(&app, "/admin/dashboard", access_token).await?;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "normal user must be denied");
    assert_eq!(body["error"]["code"], "AUTH_UNAUTHORIZED");

    Ok(())
}
