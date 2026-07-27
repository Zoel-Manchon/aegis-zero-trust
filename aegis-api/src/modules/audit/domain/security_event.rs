use serde_json::Value;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum SecurityEventType {
    RegisterSuccess,
    RegisterFailure,
    LoginSuccess,
    LoginFailure,
    MfaRequired,
    MfaSuccess,
    MfaFailure,
    RefreshSuccess,
    RefreshReplayDetected,
    SessionRevoked,
    TokenPurposeViolation,
    BruteForceLockout,
    PolicyDenied,
    ImpossibleTravel,
    DeviceFingerprintMismatch,
    CredentialStuffing,
    SessionHijack,
    PrivilegeEscalation,
}

impl SecurityEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RegisterSuccess => "REGISTER_SUCCESS",
            Self::RegisterFailure => "REGISTER_FAILURE",
            Self::LoginSuccess => "LOGIN_SUCCESS",
            Self::LoginFailure => "LOGIN_FAILURE",
            Self::MfaRequired => "MFA_REQUIRED",
            Self::MfaSuccess => "MFA_SUCCESS",
            Self::MfaFailure => "MFA_FAILURE",
            Self::RefreshSuccess => "REFRESH_SUCCESS",
            Self::RefreshReplayDetected => "REFRESH_REPLAY_DETECTED",
            Self::SessionRevoked => "SESSION_REVOKED",
            Self::TokenPurposeViolation => "TOKEN_PURPOSE_VIOLATION",
            Self::BruteForceLockout => "BRUTE_FORCE_LOCKOUT",
            Self::PolicyDenied => "POLICY_DENIED",
            Self::ImpossibleTravel => "IMPOSSIBLE_TRAVEL",
            Self::DeviceFingerprintMismatch => "DEVICE_FINGERPRINT_MISMATCH",
            Self::CredentialStuffing => "CREDENTIAL_STUFFING",
            Self::SessionHijack => "SESSION_HIJACK",
            Self::PrivilegeEscalation => "PRIVILEGE_ESCALATION",
        }
    }
}

#[derive(Debug, Clone)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SecuritySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewSecurityEvent {
    pub user_id: Option<i64>,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub session_id: Option<Uuid>,
    pub jti: Option<Uuid>,
    pub family_id: Option<Uuid>,
    pub metadata: Value,
}
