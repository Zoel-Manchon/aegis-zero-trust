mod common;

#[path = "integration/auth_login_test.rs"]
mod auth_login_test;

#[path = "integration/brute_force_test.rs"]
mod brute_force_test;

#[path = "integration/mfa_flow_test.rs"]
mod mfa_flow_test;

#[path = "integration/rbac_policy_test.rs"]
mod rbac_policy_test;

#[path = "integration/refresh_rotation_test.rs"]
mod refresh_rotation_test;

#[path = "integration/session_revocation_test.rs"]
mod session_revocation_test;

#[path = "integration/token_purpose_test.rs"]
mod token_purpose_test;

#[path = "integration/admin_security_api_test.rs"]
mod admin_security_api_test;

#[path = "integration/admin_dashboard_test.rs"]
mod admin_dashboard_test;

#[path = "integration/password_reset_test.rs"]
mod password_reset_test;

#[path = "integration/audit_chain_test.rs"]
mod audit_chain_test;
