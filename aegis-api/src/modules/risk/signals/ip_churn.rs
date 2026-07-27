//! Signal: IP churn.
//!
//! Zero-trust never assumes a session stays on one network. Two cheap heuristics:
//! the request IP differing from the IP the session was minted on, and the user
//! having logged in from several distinct IPs in the last day. Either is mildly
//! suspicious; together they add up.

use crate::modules::risk::domain::context::RiskContext;

/// 0–50 points from IP-based anomalies.
pub fn score(ctx: &RiskContext) -> u8 {
    let mut score: u8 = 0;

    // Session is bound to its original IP; a change is worth flagging.
    if ctx.original_ip != ctx.ip {
        score = score.saturating_add(25);
    }

    // Many distinct source IPs in 24h suggests travel, VPN hopping, or theft.
    if ctx.unique_ip_count_24h >= 3 {
        score = score.saturating_add(25);
    }

    score
}
