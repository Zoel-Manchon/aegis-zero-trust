//! Passkey (WebAuthn) registration + authentication, backed by `webauthn-rs`.
//!
//! The two-step ceremonies keep their in-progress state (PasskeyRegistration /
//! PasskeyAuthentication) in Redis between the begin/finish round-trips — the
//! library REQUIRES this to be persisted server-side to prevent replay. The
//! authenticator holds the private key; we only ever store the public Passkey.
use crate::{
    app_state::AppState,
    core::errors::app_error::AppError,
    modules::{
        auth::{
            application::auth_service,
            domain::auth_result::LoginResult,
            infrastructure::repositories::user_repository::UserRepository,
        },
        passkeys::{
            infrastructure::repositories::passkey_repository,
            interface::dto::passkey_dto::{PasskeyChallengeResponse, PasskeyCredentialView},
        },
    },
};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use webauthn_rs::prelude::*;

const CHALLENGE_TTL_SECONDS: usize = 300;

/// In-progress ceremony state, persisted in Redis keyed by challenge id.
#[derive(Serialize, Deserialize)]
struct RegSession {
    user_id: i64,
    reg: PasskeyRegistration,
}

#[derive(Serialize, Deserialize)]
struct AuthSession {
    user_id: i64,
    auth: PasskeyAuthentication,
}

// ---- registration ---------------------------------------------------------

pub async fn begin_registration(
    state: &AppState,
    user_id: i64,
    _user_agent: String,
    _ip: IpAddr,
) -> Result<PasskeyChallengeResponse, AppError> {
    let user = UserRepository::find_by_id(&state.pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Exclude already-registered credentials so the authenticator won't
    // double-enrol the same key.
    let exclude: Vec<CredentialID> = load_user_passkeys(state, user_id)
        .await?
        .iter()
        .map(|pk| pk.cred_id().clone())
        .collect();
    let exclude = if exclude.is_empty() { None } else { Some(exclude) };

    let webauthn = build_webauthn()?;
    let (ccr, reg) = webauthn
        .start_passkey_registration(user_handle(user_id), &user.email, &user.email, exclude)
        .map_err(|_| AppError::InternalError)?;

    let challenge_id = Uuid::new_v4().to_string();
    store_session(state, &challenge_id, &RegSession { user_id, reg }).await?;

    Ok(PasskeyChallengeResponse {
        challenge_id,
        public_key: serde_json::to_value(&ccr).map_err(|_| AppError::InternalError)?,
    })
}

pub async fn finish_registration(
    state: &AppState,
    user_id: i64,
    challenge_id: &str,
    credential: &RegisterPublicKeyCredential,
    friendly_name: Option<&str>,
    transports: &[String],
) -> Result<(), AppError> {
    let raw = take_session(state, challenge_id).await?;
    let session: RegSession = serde_json::from_str(&raw).map_err(|_| AppError::Unauthorized)?;
    if session.user_id != user_id {
        return Err(AppError::Unauthorized);
    }

    let webauthn = build_webauthn()?;
    // Real cryptographic verification: challenge, origin, rpIdHash, attestation,
    // user presence + verification, and COSE public key extraction.
    let passkey = webauthn
        .finish_passkey_registration(credential, &session.reg)
        .map_err(|_| AppError::Unauthorized)?;

    let credential_id = credential.id.clone();
    // A credential id must never be registered to two accounts.
    if passkey_repository::find_active_by_credential_id(&state.pool, &credential_id)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict);
    }

    // Persist the whole verified Passkey (public key + counter) as the stored blob.
    let blob = serde_json::to_vec(&passkey).map_err(|_| AppError::InternalError)?;
    passkey_repository::insert_passkey(
        &state.pool,
        user_id,
        &credential_id,
        &blob,
        0,
        friendly_name,
        transports,
        None,
    )
    .await?;

    Ok(())
}

// ---- authentication --------------------------------------------------------

pub async fn begin_login(
    state: &AppState,
    email: String,
    _user_agent: String,
    _ip: IpAddr,
) -> Result<PasskeyChallengeResponse, AppError> {
    let normalized = email.trim().to_lowercase();
    let user = UserRepository::find_by_email(&state.pool, &normalized).await?;

    // Generic failure whether the account is unknown or simply has no passkeys.
    let user = user.ok_or(AppError::Unauthorized)?;
    let passkeys = load_user_passkeys(state, user.id).await?;
    if passkeys.is_empty() {
        return Err(AppError::Unauthorized);
    }

    let webauthn = build_webauthn()?;
    let (rcr, auth) = webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|_| AppError::InternalError)?;

    let challenge_id = Uuid::new_v4().to_string();
    store_session(state, &challenge_id, &AuthSession { user_id: user.id, auth }).await?;

    Ok(PasskeyChallengeResponse {
        challenge_id,
        public_key: serde_json::to_value(&rcr).map_err(|_| AppError::InternalError)?,
    })
}

pub async fn finish_login(
    state: &AppState,
    challenge_id: &str,
    credential: &PublicKeyCredential,
    user_agent: String,
    ip: IpAddr,
) -> Result<LoginResult, AppError> {
    let raw = take_session(state, challenge_id).await?;
    let session: AuthSession = serde_json::from_str(&raw).map_err(|_| AppError::Unauthorized)?;

    let webauthn = build_webauthn()?;
    // Verifies the assertion signature, challenge, origin, rpIdHash, user
    // verification, and the signature counter (clone detection).
    let auth_result = webauthn
        .finish_passkey_authentication(credential, &session.auth)
        .map_err(|_| AppError::Unauthorized)?;

    let credential_id = credential.id.clone();
    let row = passkey_repository::find_active_by_credential_id(&state.pool, &credential_id)
        .await?
        .ok_or(AppError::Unauthorized)?;
    if row.user_id != session.user_id {
        return Err(AppError::Unauthorized);
    }

    // Advance the stored counter (and re-persist the updated Passkey blob) so
    // future clone-detection compares against the latest value.
    let mut passkey: Passkey =
        serde_json::from_slice(&row.public_key_cose).map_err(|_| AppError::InternalError)?;
    passkey.update_credential(&auth_result);
    let new_blob = serde_json::to_vec(&passkey).map_err(|_| AppError::InternalError)?;
    passkey_repository::update_successful_assertion(
        &state.pool,
        &credential_id,
        auth_result.counter() as i64,
        &new_blob,
    )
    .await?;

    let user = UserRepository::find_by_id(&state.pool, row.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    auth_service::issue_full_login_tokens(
        &state.pool,
        &state.jwt_keys.encoding,
        &state.refresh_secret,
        &user,
        user_agent,
        ip,
    )
    .await
}

// ---- management ------------------------------------------------------------

pub async fn list_passkeys(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<PasskeyCredentialView>, AppError> {
    let passkeys = passkey_repository::list_user_passkeys(&state.pool, user_id).await?;
    Ok(passkeys
        .into_iter()
        .map(|c| PasskeyCredentialView {
            credential_id: c.credential_id,
            friendly_name: c.friendly_name,
            transports: c.transports,
            created_at: c.created_at.to_rfc3339(),
            last_used_at: c.last_used_at.map(|dt| dt.to_rfc3339()),
        })
        .collect())
}

pub async fn revoke_passkey(
    state: &AppState,
    user_id: i64,
    credential_id: &str,
) -> Result<(), AppError> {
    passkey_repository::revoke_passkey(&state.pool, user_id, credential_id).await
}

// ---- helpers ---------------------------------------------------------------

async fn load_user_passkeys(state: &AppState, user_id: i64) -> Result<Vec<Passkey>, AppError> {
    let rows = passkey_repository::list_user_passkeys(&state.pool, user_id).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let Ok(pk) = serde_json::from_slice::<Passkey>(&row.public_key_cose) {
            out.push(pk);
        }
    }
    Ok(out)
}

async fn store_session<T: Serialize>(
    state: &AppState,
    challenge_id: &str,
    session: &T,
) -> Result<(), AppError> {
    let payload = serde_json::to_string(session).map_err(|_| AppError::InternalError)?;
    state
        .redis
        .set_ex(&challenge_key(challenge_id), &payload, CHALLENGE_TTL_SECONDS)
        .await
        .map_err(|_| AppError::InternalError)
}

async fn take_session(state: &AppState, challenge_id: &str) -> Result<String, AppError> {
    let key = challenge_key(challenge_id);
    let raw = state.redis.get_string(&key).await.map_err(|_| AppError::InternalError)?;
    state.redis.del(&key).await.map_err(|_| AppError::InternalError)?;
    raw.ok_or(AppError::Unauthorized)
}

fn challenge_key(challenge_id: &str) -> String {
    format!("passkey:challenge:{challenge_id}")
}

/// Stable per-user WebAuthn user handle derived from the account id.
fn user_handle(user_id: i64) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("aegis-user-{user_id}").as_bytes())
}

fn build_webauthn() -> Result<Webauthn, AppError> {
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin = std::env::var("WEBAUTHN_RP_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let rp_name = std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "Aegis".to_string());
    let origin = Url::parse(&rp_origin).map_err(|_| AppError::InternalError)?;
    WebauthnBuilder::new(&rp_id, &origin)
        .map_err(|_| AppError::InternalError)?
        .rp_name(&rp_name)
        .build()
        .map_err(|_| AppError::InternalError)
}
