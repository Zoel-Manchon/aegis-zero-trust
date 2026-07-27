//! `GET /me` — return the authenticated principal's identity and role.
//!
//! The access-token JWT only carries `sub`/`jti`/`purpose`; the role lives in
//! the database and is resolved per-request by `auth_middleware`. A SPA has no
//! way to learn *who* it is or whether it may see admin surfaces without this
//! endpoint, so it returns the minimum a client needs to render correctly:
//! user id, email, role, and whether MFA is enabled.
//!
//! Runs behind `auth_middleware`, so `SecurityContext` is always present.

use crate::{
    app_state::AppState,
    core::{errors::app_error::AppError, response::api_response::ApiResponse},
    modules::{
        auth::{
            infrastructure::repositories::user_repository::UserRepository,
            interface::middleware::security_context::SecurityContext,
        },
        mfa::infrastructure::repositories::mfa_repository,
    },
};

use axum::{
    Json,
    extract::{Extension, State},
};

pub async fn handler_me(
    State(state): State<AppState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user = UserRepository::find_by_id(&state.pool, ctx.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Best-effort: never fail /me just because the MFA lookup hiccups.
    let mfa_enabled = mfa_repository::find_by_user_id(&state.pool, ctx.user_id)
        .await
        .ok()
        .flatten()
        .map(|m| m.enabled)
        .unwrap_or(false);

    Ok(Json(ApiResponse::success(serde_json::json!({
        "user_id": user.id,
        "email": user.email,
        "role": user.user_role.to_string(),
        "mfa_enabled": mfa_enabled,
        "risk_score": ctx.risk_score
    }))))
}
