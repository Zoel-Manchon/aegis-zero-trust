use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::admin::security::application::security_dashboard_service,
};

use axum::{Json, extract::State};

pub async fn security_metrics_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let metrics = security_dashboard_service::security_metrics(&state.pool).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "metrics": metrics
    }))))
}
