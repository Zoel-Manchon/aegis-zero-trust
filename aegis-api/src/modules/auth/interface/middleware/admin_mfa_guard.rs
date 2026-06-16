//! Mandatory MFA for admins.
//!
//! Runs as a route layer just inside `auth_middleware` (which has already
//! inserted the `SecurityContext`). For admin callers it requires an *enabled*
//! MFA enrollment; if missing, it returns `ADMIN_MFA_REQUIRED` (403) so the
//! console can redirect the admin to enroll instead of looping on refresh.
//!
//! Non-admins pass through untouched — their role is enforced elsewhere
//! (policy engine for /admin paths, `require_admin` for the attack range).

use crate::app_state::AppState;
use crate::core::errors::app_error::AppError;
use crate::modules::auth::interface::middleware::security_context::SecurityContext;
use crate::modules::auth::models::user_model::UserRole;
use crate::modules::mfa::infrastructure::repositories::mfa_repository;

use axum::{extract::State, http::Request, middleware::Next, response::Response};

pub async fn admin_mfa_guard(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let ctx = req
        .extensions()
        .get::<SecurityContext>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    if matches!(ctx.role, UserRole::Admin) {
        let enrolled = mfa_repository::find_by_user_id(&state.pool, ctx.user_id)
            .await
            .map_err(|_| AppError::DatabaseError)?
            .map(|m| m.enabled)
            .unwrap_or(false);

        if !enrolled {
            return Err(AppError::AdminMfaRequired);
        }
    }

    Ok(next.run(req).await)
}
