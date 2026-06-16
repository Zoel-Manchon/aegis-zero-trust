use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use validator::ValidationErrors;

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
    code: String,
}

#[derive(Debug)]
pub enum AppError {
    DatabaseError,
    NotFound,
    Unauthorized,
    BadRequest,
    HashError,
    Conflict,
    InternalError,
    MfaRequired,
    StepUpRequired,
    AdminMfaRequired,
    RateLimited,
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = ?e, "database error");
        AppError::DatabaseError
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(e: argon2::password_hash::Error) -> Self {
        tracing::error!(error = ?e, "password hash error");
        AppError::HashError
    }
}

impl From<ValidationErrors> for AppError {
    fn from(e: ValidationErrors) -> Self {
        tracing::warn!(error = ?e, "validation error");
        AppError::BadRequest
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        tracing::warn!(error = ?e, "jwt error");
        AppError::Unauthorized
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message, code) = match self {
            AppError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
                "INTERNAL_ERROR",
            ),
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
                "INTERNAL_ERROR",
            ),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "Bad request", "BAD_REQUEST"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found", "NOT_FOUND"),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                "AUTH_UNAUTHORIZED",
            ),
            AppError::HashError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
                "INTERNAL_ERROR",
            ),
            AppError::Conflict => (StatusCode::CONFLICT, "Conflict", "CONFLICT"),
            AppError::MfaRequired => (StatusCode::FORBIDDEN, "MFA required", "AUTH_MFA_REQUIRED"),
            AppError::StepUpRequired => (
                StatusCode::FORBIDDEN,
                "Step-up authentication required",
                "AUTH_STEP_UP_REQUIRED",
            ),
            AppError::AdminMfaRequired => (
                StatusCode::FORBIDDEN,
                "Admin MFA enrollment required",
                "ADMIN_MFA_REQUIRED",
            ),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many attempts",
                "RATE_LIMITED",
            ),
        };

        let body = Json(ErrorResponse {
            error: ErrorBody {
                message: message.to_string(),
                code: code.to_string(),
            },
        });
        (status, body).into_response()
    }
}
