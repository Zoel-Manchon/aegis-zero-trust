use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerifyMfaRequest {
    #[validate(length(min = 6, max = 6))]
    pub code: String,
}
