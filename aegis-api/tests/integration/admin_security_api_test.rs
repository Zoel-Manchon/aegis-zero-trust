use crate::common as helpers;
use crate::common::app::spawn_test_app;
use axum::http::StatusCode;
use helpers::{
    auth::{get_with_bearer, login_user, register_user},
    fixtures::{PASSWORD, unique_email},
};

#[tokio::test]
async fn admin_can_access_security_dashboard() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    // -------------------------------------------------
    // Create fresh user for this test
    // -------------------------------------------------

    let email = unique_email("admin-api");

    let register_status = register_user(&app, &email, PASSWORD).await?;
    assert_eq!(register_status, StatusCode::OK);

    // -------------------------------------------------
    // Promote fresh user to admin
    // -------------------------------------------------

    sqlx::query(
        r#"
        UPDATE users
        SET user_role = 'admin'
        WHERE email = $1
        "#,
    )
    .bind(&email)
    .execute(&app.state.pool)
    .await?;

    // -------------------------------------------------
    // Login as admin
    // -------------------------------------------------

    let (login_status, login_body) = login_user(&app, &email, PASSWORD).await?;

    assert_eq!(login_status, StatusCode::OK);

    let access_token = login_body["data"]["access_token"]
        .as_str()
        .expect("access token missing");

    // -------------------------------------------------
    // Access admin security endpoint
    // -------------------------------------------------

    let (status, body) = get_with_bearer(&app, "/admin/security/events", access_token).await?;

    if status != StatusCode::OK {
        panic!("expected 200 OK, got {status}. body={body:#?}");
    }

    assert!(
        body["data"]["events"].is_array(),
        "security events should return data.events array, body={body:#?}"
    );

    Ok(())
}
