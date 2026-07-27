//! Attack-range HTTP handlers. Admin-only: behind auth_middleware, and each
//! handler additionally asserts the caller's role is Admin (defense in depth).

use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::{
        attack_range::application::attack_range_service::{self, LaunchReport, LaunchRequest},
        auth::{
            interface::middleware::security_context::SecurityContext, models::user_model::UserRole,
        },
        geo::origins,
    },
};
use axum::{
    Json,
    extract::{Extension, State},
};
use serde_json::json;

fn require_admin(ctx: &SecurityContext) -> Result<(), AppError> {
    if matches!(ctx.role, UserRole::Admin) {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

/// GET /attack-range/scenarios — list scenarios + attacker-origin presets.
pub async fn handler_scenarios(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&ctx)?;
    Ok(Json(ApiResponse::success(json!({
        "scenarios": attack_range_service::scenarios(),
        "origins": origins::presets(),
    }))))
}

/// POST /attack-range/launch — run a scenario from a chosen origin.
pub async fn handler_launch(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<LaunchRequest>,
) -> Result<Json<ApiResponse<LaunchReport>>, AppError> {
    require_admin(&ctx)?;
    let report = attack_range_service::launch(&state, req).await?;
    Ok(Json(ApiResponse::success(report)))
}
