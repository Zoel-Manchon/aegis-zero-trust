use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct BeginPasskeyRegistrationRequest {
    #[validate(length(min = 1, max = 80))]
    pub friendly_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FinishPasskeyRegistrationRequest {
    #[validate(length(min = 16))]
    pub challenge_id: String,
    #[validate(length(min = 8))]
    pub credential_id: String,
    #[validate(length(min = 8))]
    pub client_data_json_b64: String,
    #[validate(length(min = 8))]
    pub attestation_object_b64: String,
    pub transports: Option<Vec<String>>,
    pub friendly_name: Option<String>,
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
    #[validate(length(min = 8))]
    pub credential_id: String,
    #[validate(length(min = 8))]
    pub client_data_json_b64: String,
    #[validate(length(min = 8))]
    pub authenticator_data_b64: String,
    #[validate(length(min = 8))]
    pub signature_b64: String,
    pub user_handle_b64: Option<String>,
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
