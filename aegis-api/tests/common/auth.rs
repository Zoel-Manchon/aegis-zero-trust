use crate::common::app::TestApp;
use crate::common::app::test_addr;

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

pub async fn register_user(
    app: &TestApp,
    email: &str,
    password: &str,
) -> anyhow::Result<StatusCode> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": password
            })
            .to_string(),
        ))?;

    request.extensions_mut().insert(ConnectInfo(test_addr()));

    let response = app.router.clone().oneshot(request).await?;

    Ok(response.status())
}

pub async fn login_user(
    app: &TestApp,
    email: &str,
    password: &str,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": password
            })
            .to_string(),
        ))?;

    request.extensions_mut().insert(ConnectInfo(test_addr()));

    let response = app.router.clone().oneshot(request).await?;

    let status = response.status();
    let body = response.into_body().collect().await?.to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));

    Ok((status, json))
}

pub async fn get_with_bearer(
    app: &TestApp,
    uri: &str,
    token: &str,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())?;

    request.extensions_mut().insert(ConnectInfo(test_addr()));

    let response = app.router.clone().oneshot(request).await?;

    let status = response.status();
    let body = response.into_body().collect().await?.to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));

    Ok((status, json))
}

pub async fn post_with_bearer(
    app: &TestApp,
    uri: &str,
    token: &str,
    body_json: Value,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(body_json.to_string()))?;

    request.extensions_mut().insert(ConnectInfo(test_addr()));

    let response = app.router.clone().oneshot(request).await?;

    let status = response.status();
    let body = response.into_body().collect().await?.to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));

    Ok((status, json))
}
pub async fn post_json(
    app: &TestApp,
    uri: &str,
    body_json: Value,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body_json.to_string()))?;

    request.extensions_mut().insert(ConnectInfo(test_addr()));

    let response = app.router.clone().oneshot(request).await?;

    let status = response.status();

    let body = response.into_body().collect().await?.to_bytes();

    let json: Value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));

    Ok((status, json))
}
