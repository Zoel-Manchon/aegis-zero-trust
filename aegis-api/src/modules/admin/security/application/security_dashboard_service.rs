
// HARDENING NOTE:
// Dashboard data should become time-windowed and tenant/scope-aware. Current
// aggregate metrics are useful for MVP, but SIEM-grade dashboards need filters
// by time range, actor, IP, event type, severity, session, family_id, and rule.
use crate::modules::admin::security::domain::security_metric::SecurityMetrics;
use crate::{
    core::errors::app_error::AppError,
    modules::admin::security::{
        domain::security_event_view::SecurityEventView, infrastructure::security_query_repository,
    },
};

pub async fn security_metrics(pool: &sqlx::PgPool) -> Result<SecurityMetrics, AppError> {
    security_query_repository::security_metrics(pool)
        .await
        .map_err(|_| AppError::DatabaseError)
}

pub async fn list_security_events(
    pool: &sqlx::PgPool,
    limit: Option<i64>,
) -> Result<Vec<SecurityEventView>, AppError> {
    let limit = limit.unwrap_or(50).clamp(1, 200);

    security_query_repository::list_security_events(pool, limit)
        .await
        .map_err(|_| AppError::DatabaseError)
}
