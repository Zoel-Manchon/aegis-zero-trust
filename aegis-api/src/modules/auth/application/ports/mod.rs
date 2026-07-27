//! Ports for the auth module.
//!
//! Traits the application layer depends on; implemented by adapters in
//! `infrastructure/adapters/`. Adopted incrementally: new services (e.g.
//! password recovery) build on these directly; existing services migrate later.

pub mod session_repository;
pub mod user_repository;
