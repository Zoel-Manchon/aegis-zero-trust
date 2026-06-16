use axum::Router;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::{fs, sync::Arc};
use aegis::{
    app_state::{AppState, JwtKeys},
    core::cache::redis::RedisClient,
    modules::{
        admin::interface::routes::admin_routes, auth::interface::http::routes::auth_routes,
        mfa::interface::routes::mfa_routes,
    },
};

#[derive(Clone)]
pub struct TestApp {
    pub router: Router,
    pub state: AppState,
}

pub async fn build_test_app(pool: PgPool, redis: RedisClient) -> anyhow::Result<TestApp> {
    dotenvy::dotenv().ok();

    let refresh_secret: Arc<str> = std::env::var("REFRESH_SECRET")
        .unwrap_or_else(|_| "test-refresh-secret-minimum-32-bytes".to_string())
        .into();

    let private_key_path =
        std::env::var("JWT_PRIVATE_KEY_PATH").unwrap_or_else(|_| "private_key.pem".to_string());

    let public_key_path =
        std::env::var("JWT_PUBLIC_KEY_PATH").unwrap_or_else(|_| "public_key.pem".to_string());

    let private_key = fs::read(private_key_path)?;
    let public_key = fs::read(public_key_path)?;

    let jwt_keys = JwtKeys {
        encoding: EncodingKey::from_rsa_pem(&private_key)?,
        decoding: DecodingKey::from_rsa_pem(&public_key)?,
    };

let state = AppState::new(pool, redis, refresh_secret, Arc::new(jwt_keys));

    let router = Router::new()
        .merge(auth_routes(state.clone()))
        .merge(mfa_routes(state.clone()))
        .merge(admin_routes(state.clone()))
        .with_state(state.clone());

    Ok(TestApp { router, state })
}

pub fn test_addr() -> SocketAddr {
    "127.0.0.1:0".parse().expect("valid test socket addr")
}

pub async fn spawn_test_app() -> anyhow::Result<TestApp> {
    dotenvy::dotenv().ok();

    let pool = aegis::db::db_connect().await?;

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    let redis = aegis::core::cache::redis::RedisClient::new(&redis_url).await?;

    build_test_app(pool, redis).await
}