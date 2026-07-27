use crate::{
    core::errors::app_error::AppError,
    modules::auth::{
        domain::user::AuthUser, interface::middleware::security_context::SecurityContext,
    },
};

use axum::{extract::FromRequestParts, http::request::Parts};

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts
            .extensions
            .get::<SecurityContext>()
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthUser {
            id: ctx.user_id,
            role: ctx.role.clone(),
            jti: ctx.jti,
        })
    }
}
