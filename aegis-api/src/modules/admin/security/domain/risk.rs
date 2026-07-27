//! Risk domain primitives.
//!
//! This is the innermost layer of the hexagon: pure types and logic with **no**
//! knowledge of Postgres, Redis, Axum, or any I/O. Everything here is
//! deterministic and trivially unit-testable.
//!
//! The risk band thresholds (Low/Medium/High/Critical) live here and ONLY here.
//! Previously they were duplicated in `policy_engine::risk_decision`, which meant
//! the two could silently drift apart. Both the risk engine and the policy engine
//! now derive their behaviour from `RiskLevel`, so there is exactly one place that
//! defines what "high risk" means.

/// A bounded risk score in the inclusive range `0..=100`.
///
/// The newtype guarantees the invariant (clamped on construction) so that no
/// other code can ever hold an out-of-range score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RiskScore(u8);

impl RiskScore {
    /// Construct a score, clamping anything above 100 down to 100.
    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    /// The raw numeric value, for logging / serialization / the security context.
    pub fn value(self) -> u8 {
        self.0
    }
}

/// Coarse risk bands. The HTTP/policy layer reacts to the *band*, not the raw
/// number, so that tuning the numeric thresholds never requires touching policy
/// code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl From<RiskScore> for RiskLevel {
    /// The canonical band mapping. Change thresholds here and the whole system
    /// follows.
    fn from(score: RiskScore) -> Self {
        match score.value() {
            0..=39 => RiskLevel::Low,
            40..=69 => RiskLevel::Medium,
            70..=89 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_is_clamped_to_100() {
        assert_eq!(RiskScore::new(250).value(), 100);
    }

    #[test]
    fn band_boundaries_are_stable() {
        assert_eq!(RiskLevel::from(RiskScore::new(0)), RiskLevel::Low);
        assert_eq!(RiskLevel::from(RiskScore::new(39)), RiskLevel::Low);
        assert_eq!(RiskLevel::from(RiskScore::new(40)), RiskLevel::Medium);
        assert_eq!(RiskLevel::from(RiskScore::new(69)), RiskLevel::Medium);
        assert_eq!(RiskLevel::from(RiskScore::new(70)), RiskLevel::High);
        assert_eq!(RiskLevel::from(RiskScore::new(89)), RiskLevel::High);
        assert_eq!(RiskLevel::from(RiskScore::new(90)), RiskLevel::Critical);
        assert_eq!(RiskLevel::from(RiskScore::new(100)), RiskLevel::Critical);
    }
}