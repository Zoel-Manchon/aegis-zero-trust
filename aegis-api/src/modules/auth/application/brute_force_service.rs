use crate::core::{cache::redis::RedisClient, errors::app_error::AppError};
use crate::modules::{
    alerts::application::dispatcher::AlertDispatcher,
    audit::application::security_alerts,
};
use std::net::IpAddr;

const LOGIN_IP_WINDOW_SECONDS: usize = 60;
const LOGIN_EMAIL_WINDOW_SECONDS: usize = 300;
const MAX_LOGIN_ATTEMPTS_PER_IP: i64 = 20;
const MAX_LOGIN_ATTEMPTS_PER_EMAIL: i64 = 5;
const LOCKOUT_SECONDS: usize = 900;

pub async fn check_login_allowed(
    redis: &RedisClient,
    email: &str,
    ip: IpAddr,
) -> Result<(), AppError> {
    let ip_lock_key = format!("auth:lock:ip:{ip}");
    let email_lock_key = format!("auth:lock:email:{email}");

    if redis
        .get_i64(&ip_lock_key)
        .await
        .map_err(|_| AppError::InternalError)?
        .is_some()
    {
        return Err(AppError::RateLimited);
    }

    if redis
        .get_i64(&email_lock_key)
        .await
        .map_err(|_| AppError::InternalError)?
        .is_some()
    {
        return Err(AppError::RateLimited);
    }

    Ok(())
}

pub async fn record_failed_login(
    redis: &RedisClient,
    alerts: &AlertDispatcher,
    email: &str,
    ip: IpAddr,
) -> Result<(), AppError> {
    let ip_attempt_key = format!("auth:fail:ip:{ip}");
    let email_attempt_key = format!("auth:fail:email:{email}");

    let ip_count = redis
        .incr_with_ttl(&ip_attempt_key, LOGIN_IP_WINDOW_SECONDS)
        .await
        .map_err(|_| AppError::InternalError)?;

    let email_count = redis
        .incr_with_ttl(&email_attempt_key, LOGIN_EMAIL_WINDOW_SECONDS)
        .await
        .map_err(|_| AppError::InternalError)?;

    if ip_count > MAX_LOGIN_ATTEMPTS_PER_IP {
        let ip_lock_key = format!("auth:lock:ip:{ip}");
        redis
            .set_ex(&ip_lock_key, "1", LOCKOUT_SECONDS)
            .await
            .map_err(|_| AppError::InternalError)?;

        // Fire alert: per-IP lockout (attacker hammering many accounts).
        security_alerts::brute_force_lockout(
            alerts,
            "ip",
            &ip.to_string(),
            LOCKOUT_SECONDS as u64,
        )
        .await;
    }

    if email_count > MAX_LOGIN_ATTEMPTS_PER_EMAIL {
        let email_lock_key = format!("auth:lock:email:{email}");
        redis
            .set_ex(&email_lock_key, "1", LOCKOUT_SECONDS)
            .await
            .map_err(|_| AppError::InternalError)?;

        // Fire alert: per-email lockout (attacker targeting one account).
        security_alerts::brute_force_lockout(
            alerts,
            "email",
            email,
            LOCKOUT_SECONDS as u64,
        )
        .await;
    }

    Ok(())
}

pub async fn clear_failed_login(
    redis: &RedisClient,
    email: &str,
    ip: IpAddr,
) -> Result<(), AppError> {
    let ip_attempt_key = format!("auth:fail:ip:{ip}");
    let email_attempt_key = format!("auth:fail:email:{email}");

    let _ = redis.del(&ip_attempt_key).await;
    let _ = redis.del(&email_attempt_key).await;

    Ok(())
}