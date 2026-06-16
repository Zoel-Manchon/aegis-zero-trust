//! The decision a risk band implies.
//!
//! Kept separate from `risk.rs` so the "what score is which band" question and
//! the "what do we do about a band" question can evolve independently — but both
//! still flow from the single `RiskLevel` definition.

use crate::modules::risk::domain::risk::{RiskLevel, RiskScore};

/// What the system should do given a computed risk level.
///
/// This is the domain-level decision. The HTTP layer maps these to concrete
/// outcomes (e.g. `RequireMfa` -> `AppError::MfaRequired` -> HTTP 403), but the
/// decision itself carries no transport concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskDecision {
    /// Low risk: let the request through.
    Allow,
    /// Medium risk: require a fresh MFA assertion before proceeding.
    RequireMfa,
    /// High risk: require step-up authentication (stronger than plain MFA).
    StepUpAuth,
    /// Critical risk: refuse outright.
    Deny,
}

/// Map a score to a decision via the canonical band.
///
/// Note this goes through `RiskLevel`, so it can never disagree with any other
/// consumer of the band thresholds.
pub fn decide(score: RiskScore) -> RiskDecision {
    match RiskLevel::from(score) {
        RiskLevel::Low => RiskDecision::Allow,
        RiskLevel::Medium => RiskDecision::RequireMfa,
        RiskLevel::High => RiskDecision::StepUpAuth,
        RiskLevel::Critical => RiskDecision::Deny,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decisions_follow_bands() {
        assert_eq!(decide(RiskScore::new(10)), RiskDecision::Allow);
        assert_eq!(decide(RiskScore::new(50)), RiskDecision::RequireMfa);
        assert_eq!(decide(RiskScore::new(80)), RiskDecision::StepUpAuth);
        assert_eq!(decide(RiskScore::new(95)), RiskDecision::Deny);
    }
}