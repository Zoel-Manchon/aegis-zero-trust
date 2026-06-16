use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub message: String,
    pub code: String,
}

pub mod error_codes {
    pub const INVALID_CREDENTIALS: &str = "AUTH_INVALID_CREDENTIALS";
    pub const WEAK_PASSWORD: &str = "AUTH_WEAK_PASSWORD";
    pub const EMAIL_EXISTS: &str = "AUTH_EMAIL_EXISTS";
    pub const UNAUTHORIZED: &str = "AUTH_UNAUTHORIZED";
    pub const MFA_REQUIRED: &str = "AUTH_MFA_REQUIRED";
    pub const STEP_UP_REQUIRED: &str = "AUTH_STEP_UP_REQUIRED";
    pub const BAD_REQUEST: &str = "BAD_REQUEST";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str, code: &str) -> Self {
        Self {
            data: None,
            error: Some(ApiError {
                message: message.to_string(),
                code: code.to_string(),
            }),
        }
    }
}
