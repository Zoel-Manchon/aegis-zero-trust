//! Individual risk signals.
//!
//! Each signal is a pure `fn(&RiskContext) -> u8`. Keeping them separate makes
//! each one independently testable and lets the engine compose them by simple
//! saturating addition. To add a new signal, drop a file here and add one line
//! to the engine.

pub mod device_fingerprint;
pub mod ip_churn;
pub mod login_velocity;
pub mod session_family;
