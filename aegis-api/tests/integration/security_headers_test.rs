use crate::common::app::spawn_test_app;
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::Request,
};
use tower::ServiceExt;

/// The security headers middleware is active and emits the expected headers on
/// every response, regardless of which route was hit.
///
/// This is a smoke test that pins down ALL the hardening headers in one place.
/// If any disappears (someone removes a layer in main.rs / security_layer.rs)
/// this fails immediately.
#[tokio::test]
async fn security_headers_are_present_on_responses() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let mut request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"x@x.x","password":"x"}"#))?;
    request
        .extensions_mut()
        .insert(ConnectInfo("127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap()));

    let res = app.router.clone().oneshot(request).await?;
    let h = res.headers();

    assert!(
        h.get("x-frame-options").is_some(),
        "missing X-Frame-Options (clickjacking defense)"
    );
    assert!(
        h.get("x-content-type-options").is_some(),
        "missing X-Content-Type-Options"
    );
    assert!(
        h.get("content-security-policy").is_some(),
        "missing Content-Security-Policy"
    );
    assert!(
        h.get("referrer-policy").is_some(),
        "missing Referrer-Policy"
    );
    assert!(
        h.get("permissions-policy").is_some(),
        "missing Permissions-Policy"
    );
    Ok(())
}
