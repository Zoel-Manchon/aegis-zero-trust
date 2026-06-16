//! Infrastructure adapters implementing the auth ports.
//!
//! Thin Postgres wrappers over the existing repository SQL. The proven SQL is
//! unchanged; these only adapt it to the port traits and map errors to AppError.

pub mod pg_session_repository;
pub mod pg_user_repository;
