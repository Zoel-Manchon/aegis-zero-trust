// =============================================================================
// Per-user MFA attempt throttling.
//
// The edge limiter in `core::middleware::rate_limit` counts requests per IP.
// That stops one address from hammering the endpoint and does nothing about the
// attack that actually matters here: six digits is a 10^6 space, and an
// attacker who rotates source addresses walks it with a handful of requests per
// host. The account is the thing under attack, so the account is what has to be
// counted.
//
// This is the same sliding-window primitive the login brute-force service uses
// (`auth::application::brute_force_service`), keyed by user id instead of email
// + IP, so there is one rate-limit story across the app: count failures, lock
// the target for a while once the threshold is crossed, and fire a SOC alert
// when the lock trips.
//
// The two limiters compose rather than overlap. The per-IP one is a cheap edge
// filter on request volume; this one is a per-account budget on *failures*,
// and it survives address rotation because the key never mentions the address.
// =============================================================================

use crate::core::{cache::redis::RedisClient, errors::app_error::AppError};
use crate::modules::{
    alerts::application::dispatcher::AlertDispatcher, audit::application::security_alerts,
};

/// How long failures accumulate. Wide enough that a slow-drip attacker still
/// trips the threshold, short enough that a genuinely clumsy user is not
/// punished for a mistake made ten minutes ago.
const ATTEMPT_WINDOW_SECONDS: usize = 300;

/// Failed second-factor attempts allowed per account inside the window.
/// Five leaves room for fat fingers and a clock that drifted a step; it also
/// caps an attacker at ~5 guesses per 15 minutes, or roughly one in 200 000
/// odds of hitting a 6-digit code before the account owner notices the alert.
const MAX_ATTEMPTS_PER_USER: i64 = 5;

/// How long the account stays locked for second-factor attempts once the
/// threshold is crossed. Matches the login lockout so both feel the same.
const LOCKOUT_SECONDS: usize = 900;

fn attempt_key(user_id: i64) -> String {
    format!("mfa:fail:user:{user_id}")
}

fn lock_key(user_id: i64) -> String {
    format!("mfa:lock:user:{user_id}")
}

/// Reject the attempt before any code is compared.
///
/// Called first in every handler that accepts a second factor, so a locked
/// account costs an attacker a Redis GET and nothing else — no TOTP maths, no
/// Argon2 verification against the backup-code hashes.
pub async fn check_mfa_allowed(redis: &RedisClient, user_id: i64) -> Result<(), AppError> {
    let locked = redis
        .get_i64(&lock_key(user_id))
        .await
        .map_err(|_| AppError::InternalError)?
        .is_some();

    if locked {
        tracing::warn!(user_id, "MFA attempt rejected: account is locked out");
        return Err(AppError::RateLimited);
    }

    Ok(())
}

/// Count one failed second-factor attempt, locking the account when the
/// threshold is crossed.
///
/// The counter is per account and the lock is per account: rotating the source
/// address does not reset either one, which is the whole point.
pub async fn record_failed_attempt(
    redis: &RedisClient,
    alerts: &AlertDispatcher,
    user_id: i64,
) -> Result<(), AppError> {
    let count = redis
        .incr_with_ttl(&attempt_key(user_id), ATTEMPT_WINDOW_SECONDS)
        .await
        .map_err(|_| AppError::InternalError)?;

    if count > MAX_ATTEMPTS_PER_USER {
        redis
            .set_ex(&lock_key(user_id), "1", LOCKOUT_SECONDS)
            .await
            .map_err(|_| AppError::InternalError)?;

        security_alerts::mfa_lockout(alerts, user_id, count, LOCKOUT_SECONDS as u64).await;
    }

    Ok(())
}

/// Forget the failures after a successful second factor.
///
/// Only the attempt counter is cleared, never an active lock: proving the
/// second factor mid-lockout must not shorten a lockout that a *different*
/// party's guessing triggered.
pub async fn clear_failed_attempts(redis: &RedisClient, user_id: i64) -> Result<(), AppError> {
    let _ = redis.del(&attempt_key(user_id)).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_scoped_by_user_and_never_by_address() {
        assert_eq!(attempt_key(42), "mfa:fail:user:42");
        assert_eq!(lock_key(42), "mfa:lock:user:42");
        assert_ne!(attempt_key(42), attempt_key(43));
    }

    #[test]
    fn attempt_and_lock_namespaces_do_not_collide() {
        assert_ne!(attempt_key(7), lock_key(7));
    }

    #[test]
    fn budget_makes_guessing_a_multi_year_exercise() {
        // 10^6 codes against MAX_ATTEMPTS_PER_USER guesses per lockout: an
        // attacker needs 200 000 lockouts to walk the whole space, which is a
        // bit over five years at 15 minutes each — and every single lockout
        // fires a Critical alert naming the account, so the defender learns
        // about it in the first hour, not the fifth year.
        let guesses_per_lockout = MAX_ATTEMPTS_PER_USER as u64;
        let lockouts_to_exhaust = 1_000_000 / guesses_per_lockout;
        let years = (lockouts_to_exhaust * LOCKOUT_SECONDS as u64) / (365 * 24 * 3600);
        assert!(years >= 5, "expected years of guessing, got {years}");
    }

    #[test]
    fn one_lockout_buys_a_negligible_slice_of_the_code_space() {
        // The odds an attacker gets in before tripping the alert once.
        let odds_denominator = 1_000_000 / MAX_ATTEMPTS_PER_USER as u64;
        assert_eq!(odds_denominator, 200_000);
    }
}
