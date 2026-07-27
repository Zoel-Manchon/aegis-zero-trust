//! Signal: session family integrity.
//!
//! A refresh-token "family" should have exactly one active session at a time
//! (enforced in the DB by the `one_active_per_family` partial unique index). If
//! the engine ever observes more than one active session in a family, something
//! has gone wrong — possible token theft or a rotation race — so it scores high.

use crate::modules::risk::domain::context::RiskContext;

/// 0 or 30 points; 30 means the single-active-session invariant looks violated.
pub fn score(ctx: &RiskContext) -> u8 {
    if ctx.active_family_sessions > 1 {
        return 30;
    }

    0
}
