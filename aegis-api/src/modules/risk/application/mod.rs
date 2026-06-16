//! Risk application layer.
//!
//! Orchestration that sits between the pure domain and the I/O adapters. It owns
//! the `ports` (the traits infrastructure must satisfy), the `risk_engine` (pure
//! scoring), and the `context_builder` (gathers inputs through the ports).

pub mod context_builder;
pub mod ports;
pub mod risk_engine;
