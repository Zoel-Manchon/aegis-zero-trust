use chrono::Utc;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

use aegis::modules::risk::{
    application::risk_engine::evaluate_risk, domain::decision::RiskDecision, domain::context::RiskContext,
};

fn base_context() -> RiskContext {
    RiskContext {
        user_id: 1,
        session_id: Uuid::new_v4(),
        family_id: Uuid::new_v4(),
        jti: Uuid::new_v4(),

        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        user_agent: "Mozilla/5.0".to_string(),

        original_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        original_user_agent: "Mozilla/5.0".to_string(),

        session_created_at: Utc::now(),
        last_login_at: None,

        session_count_24h: 1,
        unique_ip_count_24h: 1,
        device_count_30d: 1,
        active_family_sessions: 1,

        request_count_60s: 1,
        policy_denial_count_10m: 0,
        mfa_failure_count_10m: 0,
    }
}

#[test]
fn low_risk_session_is_allowed() {
    let ctx = base_context();

    let evaluation = evaluate_risk(&ctx);

    assert!(evaluation.score.value() < 40);
    assert_eq!(evaluation.decision, RiskDecision::Allow);
}

#[test]
fn ip_churn_increases_risk() {
    let mut ctx = base_context();
    ctx.ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 20));

    let evaluation = evaluate_risk(&ctx);

    assert!(evaluation.score.value() >= 25);
}

#[test]
fn request_flooding_increases_risk() {
    let mut ctx = base_context();
    ctx.request_count_60s = 70;

    let evaluation = evaluate_risk(&ctx);

    assert!(evaluation.score.value() >= 30);
}

#[test]
fn mfa_failures_increase_risk() {
    let mut ctx = base_context();
    ctx.mfa_failure_count_10m = 5;

    let evaluation = evaluate_risk(&ctx);

    assert!(evaluation.score.value() >= 30);
}

#[test]
fn policy_denials_increase_risk() {
    let mut ctx = base_context();
    ctx.policy_denial_count_10m = 5;

    let evaluation = evaluate_risk(&ctx);

    assert!(evaluation.score.value() >= 25);
}

#[test]
fn device_change_increases_risk() {
    let mut ctx = base_context();
    ctx.user_agent = "Different-Agent".to_string();

    let evaluation = evaluate_risk(&ctx);

    assert!(evaluation.score.value() >= 15);
}

#[test]
fn session_family_anomaly_increases_risk() {
    let mut ctx = base_context();
    ctx.active_family_sessions = 2;

    let evaluation = evaluate_risk(&ctx);

    assert!(evaluation.score.value() >= 30);
}
