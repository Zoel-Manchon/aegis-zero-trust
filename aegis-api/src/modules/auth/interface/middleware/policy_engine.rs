use crate::{
    core::errors::app_error::AppError,
    modules::auth::interface::middleware::{
        permissions::{required_permission_for_path, role_has_permission},
        security_context::SecurityContext,
    },
};

pub enum RiskAction {
    Allow,
    RequireMfa,
    StepUpAuth,
    Deny,
}

pub fn risk_decision(score: u8) -> RiskAction {
    use crate::modules::risk::domain::risk::{RiskLevel, RiskScore};
    match RiskLevel::from(RiskScore::new(score)) {
        RiskLevel::Low      => RiskAction::Allow,
        RiskLevel::Medium   => RiskAction::RequireMfa,
        RiskLevel::High     => RiskAction::StepUpAuth,
        RiskLevel::Critical => RiskAction::Deny,
    }
}

pub fn enforce_policy(ctx: &SecurityContext, path: &str) -> Result<(), AppError> {
    if let Some(permission) = required_permission_for_path(path) {
        if !role_has_permission(&ctx.role, permission) {
            return Err(AppError::Unauthorized);
        }
    }

    match risk_decision(ctx.risk_score) {
        RiskAction::Allow => Ok(()),
        RiskAction::RequireMfa => Err(AppError::MfaRequired),
        RiskAction::StepUpAuth => Err(AppError::StepUpRequired),
        RiskAction::Deny => Err(AppError::Unauthorized),
    }
}
