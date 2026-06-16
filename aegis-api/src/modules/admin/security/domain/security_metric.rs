use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SecurityMetrics {
    pub total_events: i64,
    pub critical_events: i64,
    pub high_events: i64,
    pub refresh_replays: i64,
    pub policy_denials: i64,
    pub mfa_failures: i64,
    pub brute_force_lockouts: i64,
}
