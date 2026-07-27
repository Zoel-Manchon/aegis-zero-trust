use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

use aegis::{
    core::errors::app_error::AppError,
    modules::auth::{
        interface::middleware::{policy_engine::enforce_policy, security_context::SecurityContext},
        models::user_model::UserRole,
    },
};

fn ctx(role: UserRole, risk_score: u8) -> SecurityContext {
    SecurityContext {
        user_id: 1,
        role,
        jti: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
        user_agent: "test-agent".to_string(),
        risk_score,
    }
}

#[test]
fn normal_user_can_access_regular_route() {
    let ctx = ctx(UserRole::User, 10);

    let result = enforce_policy(&ctx, "/logout");

    assert!(result.is_ok());
}

#[test]
fn normal_user_cannot_access_admin_route() {
    let ctx = ctx(UserRole::User, 10);

    let result = enforce_policy(&ctx, "/admin/dashboard");

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[test]
fn admin_can_access_admin_route() {
    let ctx = ctx(UserRole::Admin, 10);

    let result = enforce_policy(&ctx, "/admin/dashboard");

    assert!(result.is_ok());
}

#[test]
fn medium_risk_requires_mfa() {
    let ctx = ctx(UserRole::User, 50);

    let result = enforce_policy(&ctx, "/logout");

    assert!(matches!(result, Err(AppError::MfaRequired)));
}

#[test]
fn high_risk_requires_step_up() {
    let ctx = ctx(UserRole::User, 75);

    let result = enforce_policy(&ctx, "/logout");

    assert!(matches!(result, Err(AppError::StepUpRequired)));
}

#[test]
fn critical_risk_is_denied() {
    let ctx = ctx(UserRole::User, 95);

    let result = enforce_policy(&ctx, "/logout");

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[test]
fn unknown_route_without_required_permission_uses_risk_only() {
    let ctx = ctx(UserRole::User, 10);

    let result = enforce_policy(&ctx, "/public-ish-but-protected");

    assert!(result.is_ok());
}
