use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerifyMfaRequest {
    // 6 for a TOTP code, up to 32 to leave room for a backup code typed
    // with or without its dash and any stray whitespace.
    #[validate(length(min = 6, max = 32))]
    pub code: String,
}
