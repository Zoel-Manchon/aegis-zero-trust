use crate::common::app::spawn_test_app;
use crate::common::fixtures::{unique_email, PASSWORD};
use crate::common::auth::{login_user, register_user};
use aegis::modules::audit::infrastructure::repositories::chained_audit_repository::verify_chain;
use std::sync::Mutex;

// Serializes the chain tests so they never run concurrently with each other.
// The audit chain is a single global structure over a shared test DB, so two
// tests verifying/tampering it at once would interfere. This guard makes them
// run one at a time. (Other tests may still append events concurrently, which
// is fine — appends keep the chain valid; only tampering breaks it.)
static CHAIN_TEST_GUARD: Mutex<()> = Mutex::new(());

/// Real activity produces a valid, internally consistent hash chain.
#[tokio::test]
async fn audit_chain_verifies_after_activity() -> anyhow::Result<()> {
    let _guard = CHAIN_TEST_GUARD.lock().unwrap();

    let app = spawn_test_app().await?;
    let email = unique_email("chain");
    register_user(&app, &email, PASSWORD).await?;
    login_user(&app, &email, PASSWORD).await?;
    login_user(&app, &email, "wrong-password").await?;

    let result = verify_chain(&app.state.pool).await?;
    assert!(result.verified, "chain should verify clean: {:?}", result.reason);
    assert!(result.events_checked > 0, "should have checked events");
    Ok(())
}

/// Tampering with a stored event is DETECTED, then the row is RESTORED so the
/// shared chain is left consistent for other tests.
#[tokio::test]
async fn audit_chain_detects_tampering() -> anyhow::Result<()> {
    let _guard = CHAIN_TEST_GUARD.lock().unwrap();

    let app = spawn_test_app().await?;
    let email = unique_email("chain-tamper");
    register_user(&app, &email, PASSWORD).await?;
    login_user(&app, &email, PASSWORD).await?;
    login_user(&app, &email, PASSWORD).await?;

    let target: Option<(i64, String)> = sqlx::query_as(
        "SELECT seq, severity FROM security_events WHERE seq IS NOT NULL ORDER BY seq DESC LIMIT 1",
    )
    .fetch_optional(&app.state.pool)
    .await?;

    let Some((seq, original_severity)) = target else {
        return Ok(());
    };

    let tampered_value = if original_severity == "CRITICAL" { "INFO" } else { "CRITICAL" };
    sqlx::query("UPDATE security_events SET severity = $1 WHERE seq = $2")
        .bind(tampered_value).bind(seq)
        .execute(&app.state.pool).await?;

    let after = verify_chain(&app.state.pool).await?;
    let detected = !after.verified;

    // Restore so the chain heals (restoring severity restores the original hash).
    sqlx::query("UPDATE security_events SET severity = $1 WHERE seq = $2")
        .bind(&original_severity).bind(seq)
        .execute(&app.state.pool).await?;

    assert!(detected, "tampering must be detected");
    assert!(after.broken_at_seq.is_some(), "should identify a broken seq");
    Ok(())
}