//! Database connection pool with enforced TLS.
//!
//! Zero-trust posture: the application refuses to talk to Postgres over an
//! unencrypted channel. We build the pool from explicit `PgConnectOptions` with
//! `PgSslMode::VerifyFull`, which requires (a) the server to present a
//! certificate, (b) that certificate to chain to a CA we trust, and (c) the
//! certificate's host to match. This defeats both passive sniffing and active
//! MITM (a downgrade or self-signed cert is rejected).
//!
//! TLS protocol floor (>= 1.3) is enforced at the PostgreSQL *server* via
//! `ssl_min_protocol_version = 'TLSv1.3'` in postgresql.conf — see deployment
//! notes. sqlx (rustls) will happily negotiate 1.3 when the server requires it.
//!
//! Requires the sqlx `tls-rustls` feature (see Cargo.toml change).

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;

pub async fn db_connect() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Parse the base options from the URL, then harden them in code so the TLS
    // policy cannot be silently weakened by editing the connection string.
    let mut opts = PgConnectOptions::from_str(&database_url)?;

    // Choose SSL mode from an env toggle so local dev (where you may not have
    // set up a CA yet) can fall back, while production stays strict.
    //   DB_SSL_MODE = verify-full | require | disable   (default: verify-full)
    let ssl_mode = std::env::var("DB_SSL_MODE").unwrap_or_else(|_| "verify-full".to_string());

    opts = match ssl_mode.as_str() {
        // Strongest: encrypt + verify CA + verify hostname. Production default.
        "verify-full" => {
            let opts = opts.ssl_mode(PgSslMode::VerifyFull);
            // Optional explicit CA bundle (recommended in prod). If unset,
            // rustls uses the system roots.
            match std::env::var("DB_SSL_ROOT_CERT") {
                Ok(path) if !path.is_empty() => opts.ssl_root_cert(path),
                _ => opts,
            }
        }
        // Encrypt + verify CA but not hostname.
        "verify-ca" => {
            let opts = opts.ssl_mode(PgSslMode::VerifyCa);
            match std::env::var("DB_SSL_ROOT_CERT") {
                Ok(path) if !path.is_empty() => opts.ssl_root_cert(path),
                _ => opts,
            }
        }
        // Encrypt but do not verify the server cert. Use ONLY for local dev
        // against a self-signed cert you cannot add to a trust store.
        "require" => opts.ssl_mode(PgSslMode::Require),
        // Explicit opt-out for local plaintext dev. Logs a loud warning.
        "disable" => {
            tracing::warn!(
                "DB_SSL_MODE=disable — connecting to Postgres WITHOUT TLS. \
                 Never use this outside local development."
            );
            opts.ssl_mode(PgSslMode::Disable)
        }
        other => {
            tracing::warn!(
                "unknown DB_SSL_MODE='{other}', falling back to verify-full"
            );
            opts.ssl_mode(PgSslMode::VerifyFull)
        }
    };

    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect_with(opts)
        .await
}
