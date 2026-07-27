//! The risk engine: pure scoring orchestration.
//!
//! Given a fully-populated `RiskContext`, sum the individual signals plus a few
//! inline temporal/velocity rules, clamp to a `RiskScore`, and derive a
//! `RiskDecision`. This function is pure and synchronous — no I/O — so it is
//! exhaustively unit-testable (see `tests/unit/risk_engine_test.rs`).
//!
//! Gathering the context (the I/O part) is the builder's job; this module only
//! does arithmetic.

use crate::modules::risk::domain::{
    context::RiskContext,
    decision::{decide, RiskDecision},
    risk::RiskScore,
};
use crate::modules::risk::signals::{device_fingerprint, ip_churn, login_velocity, session_family};

/// The output of an evaluation: the numeric score and the decision it implies.
pub struct RiskEvaluation {
    pub score: RiskScore,
    pub decision: RiskDecision,
}

/// Score a request and decide what to do about it.
pub fn evaluate_risk(ctx: &RiskContext) -> RiskEvaluation {
    let mut score: u8 = 0;

    // --- composable signals (each a pure fn over the context) ---
    score = score.saturating_add(ip_churn::score(ctx));
    score = score.saturating_add(device_fingerprint::score(ctx));
    score = score.saturating_add(login_velocity::score(ctx));
    score = score.saturating_add(session_family::score(ctx));

    // --- session-age heuristics ---
    let age_hours = (chrono::Utc::now() - ctx.session_created_at).num_hours();
    if age_hours < 1 {
        // Brand-new sessions are slightly riskier (just minted, less proven).
        score = score.saturating_add(10);
    }
    if age_hours > 24 * 30 {
        // Very old sessions (>30d) are mildly suspicious if still alive.
        score = score.saturating_add(5);
    }

    // --- request burst (60s window from the signal store) ---
    if ctx.request_count_60s > 60 {
        score = score.saturating_add(30);
    } else if ctx.request_count_60s > 30 {
        score = score.saturating_add(15);
    }

    // --- repeated MFA failures (10m window) ---
    if ctx.mfa_failure_count_10m >= 5 {
        score = score.saturating_add(30);
    } else if ctx.mfa_failure_count_10m >= 3 {
        score = score.saturating_add(15);
    }

    // --- repeated policy denials (10m window) ---
    if ctx.policy_denial_count_10m >= 5 {
        score = score.saturating_add(30);
    } else if ctx.policy_denial_count_10m >= 3 {
        score = score.saturating_add(15);
    }

    let score = RiskScore::new(score);
    let decision = decide(score);

    RiskEvaluation { score, decision }
}
