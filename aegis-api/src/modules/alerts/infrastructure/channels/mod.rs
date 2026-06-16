//! Alert delivery channel adapters.
//!
//! - `LogChannel`         -> always-on tracing sink (safe default)
//! - `EmailChannel`       -> SMTP (stub seam until lettre + Vault are wired)
//! - `RedisStreamChannel` -> pushes latest alert to Redis for the SSE stream

pub mod email_channel;
pub mod log_channel;
pub mod websocket_channel;
pub mod broadcast_channel;
