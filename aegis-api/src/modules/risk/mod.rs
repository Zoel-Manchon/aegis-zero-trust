//! Risk module.
//!
//! Hexagonal layout:
//!   domain/         pure types + scoring inputs (no I/O)
//!   application/    ports (traits) + engine + context builder
//!   infrastructure/ adapters implementing the ports (Redis, Postgres)
//!   signals/        individual pure scoring functions
//!
//! Dependency rule: domain <- application <- infrastructure. Nothing inner
//! imports anything outer.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod signals;
