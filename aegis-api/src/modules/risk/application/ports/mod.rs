//! Ports for the risk module.
//!
//! A "port" is a trait that the application layer owns and depends on, and that
//! an infrastructure adapter implements. This is the boundary that makes the
//! hexagon testable: swap the Redis/Postgres adapters for in-memory fakes and
//! the application logic doesn't notice.

pub mod history_store;
pub mod signal_store;
