//! HTTP handlers for email verification.
//!
//! Both endpoints return a generic message and never leak whether the email
//! exists or whether the token was actually valid beyond a 401 vs 200.

use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use validator::Validate;

use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::auth::{
        application::email_verification_service,
        interface::http::dto::verify_email_dto::{VerifyConfirmDto, VerifyRequestDto},
    },
};

/// POST /verify-email/request — anti-enumeration: always 200 with same body.
pub async fn handler_request_email_verification(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequestDto>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), AppError> {
    payload.validate().map_err(|_| AppError::BadRequest)?;

    email_verification_service::request_verification(&state, &payload.email).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(json!({
            "message": "If that email exists and is not already verified, a verification link has been sent."
        }))),
    ))
}

/// POST /verify-email/confirm — consumes a token (single-use). 200 on success,
/// 401 on invalid/expired/already-used. Body shape identical across failures.
pub async fn handler_confirm_email_verification(
    State(state): State<AppState>,
    Json(payload): Json<VerifyConfirmDto>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), AppError> {
    payload.validate().map_err(|_| AppError::BadRequest)?;

    email_verification_service::perform_verification(&state, &payload.token).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(json!({
            "message": "Email verified successfully."
        }))),
    ))
}