//! Risk infrastructure adapters.
//!
//! Concrete implementations of the application ports:
//! - `RedisRiskSignalStore`  -> `RiskSignalStore`  (short-window counters)
//! - `PgRiskHistoryStore`     -> `RiskHistoryStore` (durable aggregates)
//!
//! These are the only files in the risk module that know about Redis or sqlx.

pub mod pg_history_store;
pub mod redis_signal_store;
