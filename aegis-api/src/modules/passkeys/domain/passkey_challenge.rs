use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasskeyChallengePurpose {
    Registration,
    Authentication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPasskeyChallenge {
    pub user_id: Option<i64>,
    pub email: Option<String>,
    pub challenge: String,
    pub purpose: PasskeyChallengePurpose,
    pub user_agent: String,
    pub ip: String,
}
