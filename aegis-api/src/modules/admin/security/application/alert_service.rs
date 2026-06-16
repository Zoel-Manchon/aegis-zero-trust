use crate::{
    core::errors::app_error::AppError,
    modules::admin::security::{
        domain::security_alert::SecurityAlert, infrastructure::security_query_repository,
    },
};

pub async fn derived_security_alerts(pool: &sqlx::PgPool) -> Result<Vec<SecurityAlert>, AppError> {
    security_query_repository::derived_security_alerts(pool)
        .await
        .map_err(|_| AppError::DatabaseError)
}
