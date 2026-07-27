// ============================================================================
// app_state.rs
//
// What changed and why:
//   The risk module now depends on two PORTS (traits), not on concrete Redis /
//   Postgres calls. AppState constructs the two ADAPTERS once at startup and
//   holds them behind Arc<dyn Trait> so every handler/middleware shares one
//   instance and the wiring lives in exactly one place.
//
//   We keep `pool` and `redis` as well, because the rest of the app (auth, mfa,
//   audit, admin) still uses them directly. Those modules will be migrated to
//   their own ports in later steps; until then both styles coexist safely.
// ============================================================================

// ============================================================================
// app_state.rs — holds shared state: pool, redis, JWT keys, risk ports, and
// the alert dispatcher (notification delivery).
// ============================================================================

use crate::core::cache::redis::RedisClient;
use crate::modules::alerts::application::channel::AlertChannel;
use crate::modules::alerts::application::dispatcher::AlertDispatcher;
use crate::modules::alerts::domain::alert::Alert;
use crate::modules::alerts::infrastructure::channels::{
    broadcast_channel::BroadcastAlertChannel, email_channel::EmailChannel, log_channel::LogChannel,
    websocket_channel::RedisStreamChannel,
};
use crate::modules::risk::application::ports::{
    history_store::RiskHistoryStore, signal_store::RiskSignalStore,
};
use crate::modules::risk::infrastructure::{
    pg_history_store::PgRiskHistoryStore, redis_signal_store::RedisRiskSignalStore,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: RedisClient,
    pub refresh_secret: Arc<str>,
    pub jwt_keys: Arc<JwtKeys>,

    // --- risk module ports ---
    pub risk_signals: Arc<dyn RiskSignalStore>,
    pub risk_history: Arc<dyn RiskHistoryStore>,

    // --- alerts: notification delivery ---
    pub alerts: AlertDispatcher,
    pub alert_bus: broadcast::Sender<Alert>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        redis: RedisClient,
        refresh_secret: Arc<str>,
        jwt_keys: Arc<JwtKeys>,
    ) -> Self {
        let risk_signals: Arc<dyn RiskSignalStore> =
            Arc::new(RedisRiskSignalStore::new(redis.clone()));
        let risk_history: Arc<dyn RiskHistoryStore> =
            Arc::new(PgRiskHistoryStore::new(pool.clone()));

        let from = std::env::var("ALERT_FROM_EMAIL")
            .unwrap_or_else(|_| "no-reply@aegis.local".to_string());
        let (alert_bus, _rx) = broadcast::channel::<Alert>(512);
        let channels: Vec<Arc<dyn AlertChannel>> = vec![
            Arc::new(LogChannel),
            Arc::new(EmailChannel::new(from)),
            Arc::new(RedisStreamChannel::new(redis.clone())),
            Arc::new(BroadcastAlertChannel::new(alert_bus.clone())),
        ];
        let alerts = AlertDispatcher::new(channels);

        Self {
            pool,
            redis,
            refresh_secret,
            jwt_keys,
            risk_signals,
            risk_history,
            alerts,
            alert_bus,
        }
    }
}

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}