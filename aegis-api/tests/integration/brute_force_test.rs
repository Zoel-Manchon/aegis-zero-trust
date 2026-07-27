use crate::common as helpers;
use axum::http::StatusCode;
use helpers::{
    auth::{login_user, register_user},
    fixtures::{PASSWORD, unique_email},
};

use crate::common::app::spawn_test_app;
#[tokio::test]
async fn repeated_failed_logins_trigger_rate_limit() -> anyhow::Result<()> {
    let app = spawn_test_app().await?;

    let email = unique_email("bruteforce");

    register_user(&app, &email, PASSWORD).await?;

    let mut last_status = StatusCode::OK;

    for attempt in 1..=8 {
        let (status, _body) = login_user(&app, &email, &format!("WrongPassword{attempt}!")).await?;

        last_status = status;
    }

    assert_eq!(last_status, StatusCode::TOO_MANY_REQUESTS);

    Ok(())
}
