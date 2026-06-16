//! DTOs for the email verification endpoints.

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyRequestDto {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyConfirmDto {
    #[validate(length(min = 16, max = 512))]
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct GenericMessage {
    pub message: String,
}
