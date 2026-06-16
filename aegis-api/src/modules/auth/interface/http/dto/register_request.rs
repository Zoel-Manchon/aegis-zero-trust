use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email, length(max = 254))]
    pub email: String,

    #[validate(length(min = 12, max = 1024))]
    pub password: String,
}
