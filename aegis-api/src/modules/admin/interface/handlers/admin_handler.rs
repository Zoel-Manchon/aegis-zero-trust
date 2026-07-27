use crate::{
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::auth::interface::middleware::security_context::SecurityContext,
};

use axum::{Json, extract::Extension};

pub async fn admin_dashboard_handler(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "Admin dashboard access granted",
        "user_id": ctx.user_id,
        "role": ctx.role.to_string(),
        "risk_score": ctx.risk_score
    }))))
}
