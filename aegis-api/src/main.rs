use aegis::{
    app_state::{AppState, JwtKeys},
    core::{cache::redis::RedisClient, middleware::security_layer},
    db::db_connect,
    modules::{
        admin::interface::routes::admin_routes,
        attack_range::interface::routes::attack_range_routes,
        auth::interface::http::routes::auth_routes,
        mfa::interface::routes::mfa_routes, passkeys::interface::routes::passkey_routes,
    },
};

use axum::Router;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::{fs, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

    let redis = RedisClient::new(&redis_url).await?;
    let pool = db_connect().await?;

    let refresh_secret: Arc<str> = std::env::var("REFRESH_SECRET")
        .expect("REFRESH_SECRET must be set")
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

    // Build the route tree.
    let app = Router::new()
        .merge(auth_routes(state.clone()))
        .merge(mfa_routes(state.clone()))
        .merge(passkey_routes(state.clone()))
        .merge(admin_routes(state.clone()))
        .merge(attack_range_routes(state.clone()))
        .with_state(state);

    // Apply the global security layer (headers, CORS, body cap, timeout).
    // MUST be applied BEFORE into_make_service_with_connect_info.
    let app = security_layer::apply(app)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr: SocketAddr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()?;

    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
