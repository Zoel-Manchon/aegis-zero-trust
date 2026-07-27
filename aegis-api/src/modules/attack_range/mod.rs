//! Attack range — an operator-driven adversary for the SOC.
//!
//! Admins pick an attacker ORIGIN (which drives GeoIP) and a SCENARIO, and the
//! range writes the resulting security events (attributed to the chosen origin
//! and the victim) into the audit log and dispatches alerts. Launching from two
//! distant origins in a row trips impossible-travel detection — the same loop
//! the reference lab demonstrates, but server-side (no header spoofing).
pub mod application;
pub mod interface;
