//! Risk domain module.
//!
//! Pure, dependency-free core: score/level types, the decision mapping, and the
//! `RiskContext` value object. Nothing in here imports from `application/` or
//! `infrastructure/` — dependencies only point *inward*.

pub mod context;
pub mod decision;
pub mod risk;
