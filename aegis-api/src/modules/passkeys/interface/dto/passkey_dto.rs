use serde::{Deserialize, Serialize};
use validator::Validate;
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential};

#[derive(Debug, Deserialize, Validate)]
pub struct BeginPasskeyRegistrationRequest {
    #[validate(length(min = 1, max = 80))]
    pub friendly_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FinishPasskeyRegistrationRequest {
    #[validate(length(min = 16))]
    pub challenge_id: String,
    /// The raw WebAuthn attestation credential produced by `navigator.credentials.create()`.
    pub credential: RegisterPublicKeyCredential,
    #[validate(length(min = 1, max = 80))]
    pub friendly_name: Option<String>,
    pub transports: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BeginPasskeyLoginRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FinishPasskeyLoginRequest {
    #[validate(length(min = 16))]
    pub challenge_id: String,
    /// The raw WebAuthn assertion credential produced by `navigator.credentials.get()`.
    pub credential: PublicKeyCredential,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeletePasskeyRequest {
    #[validate(length(min = 8))]
    pub credential_id: String,
}

#[derive(Debug, Serialize)]
pub struct PasskeyChallengeResponse {
    pub challenge_id: String,
    pub public_key: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct PasskeyCredentialView {
    pub credential_id: String,
    pub friendly_name: Option<String>,
    pub transports: Vec<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}
