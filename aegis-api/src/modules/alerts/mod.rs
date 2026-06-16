//! Alerts module — notification delivery (push side).
//!
//! Hexagonal layout:
//!   domain/         the deliverable Alert value object
//!   application/    the AlertChannel port + AlertDispatcher
//!   infrastructure/ channel adapters (log, email, redis-stream)
//!
//! This complements `admin::security` (the read/derive side of alerts shown on
//! the dashboard). Here we *deliver* alerts when events fire; there we *query*
//! aggregate alerts for display.

pub mod application;
pub mod domain;
pub mod infrastructure;
