cd ~/projects/aegis-api
cat > tests/integration/password_reset_test.rs << 'RUSTEOF'
use crate::common as helpers;
use crate::common::app::spawn_test_app;
use crate::common::fixtures::{unique_email, PASSWORD};
use axum::http::StatusCode;
use helpers::auth::{login_user, post_json, register_user};
use serde_json::json;
use uuid::Uuid;

/// Local SHA-256 hex helper (matches the service's hash_token).
fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(input.as_bytes()))
}

/// A fresh, unique raw token per call so repeated test runs never collide on the
/// UNIQUE(token_hash) constraint.
fn fresh_token() -> String {
    format!("reset_{}", Uuid::new_v4().simple())
}

/// Insert a known reset token row for a user and return the raw token.
async fn seed_token(pool: &sqlx::PgPool, user_id: i64) -> anyhow::Result<String> {
    let raw = fresh_token();
    sqlx::query(
        r#"
        INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, now() + interval '30 minutes')
        "#,
    )
    .bind(user_id)
    .bind(sha256_hex(&raw))
    .execute(pool)
    .await?;
    Ok(raw)
}

async fn user_id_for(pool: &sqlx::PgPool, email: &str) -> anyhow::Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Full happy path: forgot -> seed token -> reset -> old fails, new works.
#[tokio::test]
async fn password_reset_full_flow() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("pwreset");
    assert_eq!(register_user(&app, &email, PASSWORD).await?, StatusCode::OK);

    let (forgot_status, forgot_body) =
        post_json(&app, "/password/forgot", json!({ "email": email })).await?;
    assert_eq!(forgot_status, StatusCode::OK);
    assert!(forgot_body["data"].as_str().unwrap().contains("reset link"));

    let uid = user_id_for(&app.state.pool, &email).await?;
    let raw_token = seed_token(&app.state.pool, uid).await?;

    let new_password = "BrandNewPassword456!";
    let (reset_status, _) = post_json(
        &app,
        "/password/reset",
        json!({ "token": raw_token, "new_password": new_password }),
    )
    .await?;
    assert_eq!(reset_status, StatusCode::OK);

    let (old_login, _) = login_user(&app, &email, PASSWORD).await?;
    assert_eq!(old_login, StatusCode::UNAUTHORIZED);

    let (new_login, new_body) = login_user(&app, &email, new_password).await?;
    assert_eq!(new_login, StatusCode::OK);
    assert!(new_body["data"]["access_token"].is_string());

    Ok(())
}

/// A used token cannot be reused.
#[tokio::test]
async fn reset_token_is_single_use() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("pwreset-single");
    register_user(&app, &email, PASSWORD).await?;

    let uid = user_id_for(&app.state.pool, &email).await?;
    let raw_token = seed_token(&app.state.pool, uid).await?;

    let (first, _) = post_json(
        &app,
        "/password/reset",
        json!({ "token": raw_token, "new_password": "FirstNewPass123!" }),
    )
    .await?;
    assert_eq!(first, StatusCode::OK);

    let (second, _) = post_json(
        &app,
        "/password/reset",
        json!({ "token": raw_token, "new_password": "SecondNewPass123!" }),
    )
    .await?;
    assert_eq!(second, StatusCode::UNAUTHORIZED);

    Ok(())
}

/// Forgot-password for an unknown email returns the same response (no enumeration).
#[tokio::test]
async fn forgot_password_does_not_enumerate() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let unknown = unique_email("does-not-exist");
    let (status, body) =
        post_json(&app, "/password/forgot", json!({ "email": unknown })).await?;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].as_str().unwrap().contains("reset link"));

    Ok(())
}
RUSTEOF
echo "rewritten"