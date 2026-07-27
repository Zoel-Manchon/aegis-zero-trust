use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::admin::security::application::alert_service,
};

use axum::{Json, extract::State};

pub async fn security_alerts_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let alerts = alert_service::derived_security_alerts(&state.pool).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "alerts": alerts
    }))))
}
