use crate::common::app::spawn_test_app;
use crate::common::auth::register_user;
use crate::common::fixtures::{unique_email, PASSWORD};
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tower::ServiceExt;

fn hash_token(raw: &str) -> String {
    hex::encode(Sha256::digest(raw.as_bytes()))
}

async fn post_json(app: &crate::common::app::TestApp, path: &str, body: Value) -> (u16, Value) {
    let mut req = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    req.extensions_mut().insert(ConnectInfo(
        "127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap(),
    ));
    let res = app.router.clone().oneshot(req).await.unwrap();
    let status = res.status().as_u16();
    let bytes = axum::body::to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

/// /verify-email/request returns 200 with the SAME body for both a known and an
/// unknown email — no enumeration leak.
#[tokio::test]
async fn verify_email_request_does_not_enumerate() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;
    let email = unique_email("verif");
    register_user(&app, &email, PASSWORD).await?;

    let (s1, b1) = post_json(&app, "/verify-email/request", serde_json::json!({"email": email})).await;
    let (s2, b2) = post_json(
        &app,
        "/verify-email/request",
        serde_json::json!({"email": unique_email("ghost")}),
    )
    .await;

    assert_eq!(s1, 200);
    assert_eq!(s2, 200);
    assert_eq!(b1, b2, "responses for known and unknown emails must be identical");
    Ok(())
}

/// Full flow: registering generates a token (we read its hash from the DB),
/// re-issue via /verify-email/request, then confirm. After confirmation the
/// user has email_verified_at set.
#[tokio::test]
async fn verify_email_full_flow() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;
    let email = unique_email("verif-flow");
    register_user(&app, &email, PASSWORD).await?;

    // Request a verification email — but we can't read the raw token from the
    // API (intentional), so we seed a known one directly to exercise /confirm.
    // This mirrors how the password_reset_test handles the same constraint.
    let user_row: (i64,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&app.state.pool)
        .await?;
    let user_id = user_row.0;

    let raw_token = "test-verify-token-aaaaaaaaaaaaaaaa";
    let token_hash = hash_token(raw_token);
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    sqlx::query(
        "INSERT INTO email_verification_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&app.state.pool)
    .await?;

    let (status, _body) = post_json(
        &app,
        "/verify-email/confirm",
        serde_json::json!({"token": raw_token}),
    )
    .await;
    assert_eq!(status, 200, "confirm should succeed");

    // Verified column flipped.
    let (verified_at,): (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT email_verified_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&app.state.pool)
            .await?;
    assert!(verified_at.is_some(), "email_verified_at must be set after confirm");

    Ok(())
}

/// Single-use: a successfully-used token cannot be replayed.
#[tokio::test]
async fn verify_email_token_is_single_use() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;
    let email = unique_email("verif-once");
    register_user(&app, &email, PASSWORD).await?;

    let (user_id,): (i64,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&app.state.pool)
        .await?;

    let raw_token = "test-verify-once-token-bbbbbbbbbbbb";
    let token_hash = hash_token(raw_token);
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    sqlx::query(
        "INSERT INTO email_verification_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&app.state.pool)
    .await?;

    let (s1, _) = post_json(&app, "/verify-email/confirm", serde_json::json!({"token": raw_token})).await;
    assert_eq!(s1, 200, "first use should succeed");

    let (s2, _) = post_json(&app, "/verify-email/confirm", serde_json::json!({"token": raw_token})).await;
    assert_eq!(s2, 401, "replay must be rejected");

    Ok(())
}
