//! Signal: login velocity.
//!
//! How many sessions the user has created in the last 24h. A handful is normal
//! (phone + laptop + re-logins); a flood suggests automation or credential
//! stuffing succeeding repeatedly.

use crate::modules::risk::domain::context::RiskContext;

/// 0–40 points scaled by 24h session count.
pub fn score(ctx: &RiskContext) -> u8 {
    match ctx.session_count_24h {
        0..=2 => 0,
        3..=5 => 10,
        6..=10 => 25,
        _ => 40,
    }
}
