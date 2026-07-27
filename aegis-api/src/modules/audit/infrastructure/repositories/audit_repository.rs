use crate::modules::audit::domain::security_event::NewSecurityEvent;

pub async fn insert_security_event(
    pool: &sqlx::PgPool,
    event: NewSecurityEvent,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO security_events (
            user_id,
            event_type,
            severity,
            ip_address,
            user_agent,
            session_id,
            jti,
            family_id,
            metadata
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9
        )
        "#,
    )
    .bind(event.user_id)
    .bind(event.event_type.as_str())
    .bind(event.severity.as_str())
    .bind(event.ip_address)
    .bind(event.user_agent)
    .bind(event.session_id)
    .bind(event.jti)
    .bind(event.family_id)
    .bind(event.metadata)
    .execute(pool)
    .await?;

    Ok(())
}
