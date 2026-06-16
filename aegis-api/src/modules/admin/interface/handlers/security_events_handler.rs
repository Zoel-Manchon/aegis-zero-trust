use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::admin::security::application::security_dashboard_service,
};

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SecurityEventsQuery {
    pub limit: Option<i64>,
}

pub async fn security_events_handler(
    State(state): State<AppState>,
    Query(query): Query<SecurityEventsQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let events = security_dashboard_service::list_security_events(&state.pool, query.limit).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "events": events
    }))))
}
