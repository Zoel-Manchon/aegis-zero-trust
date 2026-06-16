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
            domain::passkey_challenge::{PasskeyChallengePurpose, StoredPasskeyChallenge},
            infrastructure::repositories::passkey_repository,
            interface::dto::passkey_dto::{PasskeyChallengeResponse, PasskeyCredentialView},
        },
    },
};
use serde_json::json;
use std::net::IpAddr;
use uuid::Uuid;

const CHALLENGE_TTL_SECONDS: usize = 300;

pub async fn begin_registration(
    state: &AppState,
    user_id: i64,
    user_agent: String,
    ip: IpAddr,
) -> Result<PasskeyChallengeResponse, AppError> {
    let challenge_id = Uuid::new_v4().to_string();
    let challenge = Uuid::new_v4().to_string();
    let stored = StoredPasskeyChallenge {
        user_id: Some(user_id),
        email: None,
        challenge: challenge.clone(),
        purpose: PasskeyChallengePurpose::Registration,
        user_agent,
        ip: ip.to_string(),
    };

    state.redis.set_ex(
        &challenge_key(&challenge_id),
        &serde_json::to_string(&stored).map_err(|_| AppError::InternalError)?,
        CHALLENGE_TTL_SECONDS,
    ).await.map_err(|_| AppError::InternalError)?;

    // WebAuthn frontend payload. The final cryptographic verification must be
    // wired to a WebAuthn crate before production; this API shape is ready for it.
    Ok(PasskeyChallengeResponse {
        challenge_id,
        public_key: json!({
            "challenge": challenge,
            "rp": { "name": rp_name(), "id": rp_id() },
            "user": { "id": user_id.to_string(), "name": user_id.to_string(), "displayName": user_id.to_string() },
            "pubKeyCredParams": [
                { "type": "public-key", "alg": -7 },
                { "type": "public-key", "alg": -257 }
            ],
            "authenticatorSelection": {
                "residentKey": "preferred",
                "userVerification": "required"
            },
            "attestation": "none",
            "timeout": 300000
        }),
    })
}

pub async fn finish_registration(
    state: &AppState,
    user_id: i64,
    challenge_id: &str,
    credential_id: &str,
    attestation_object_b64: &str,
    friendly_name: Option<&str>,
    transports: &[String],
) -> Result<(), AppError> {
    let stored = take_challenge(state, challenge_id).await?;

    if !matches!(stored.purpose, PasskeyChallengePurpose::Registration) || stored.user_id != Some(user_id) {
        return Err(AppError::Unauthorized);
    }

    // HARDENING TODO before enabling in production:
    // Verify clientDataJSON.challenge, origin, rpIdHash, attestationObject,
    // user presence, user verification, alg allowlist, and extract the COSE
    // public key + initial sign counter using `webauthn-rs` or equivalent.
    // This placeholder stores the attestation blob as a compile-time bridge so
    // the repository/routes/tests can be built around the correct domain model.
    let public_key_cose_placeholder = attestation_object_b64.as_bytes();

    passkey_repository::insert_passkey(
        &state.pool,
        user_id,
        credential_id,
        public_key_cose_placeholder,
        0,
        friendly_name,
        transports,
        None,
    ).await?;

    Ok(())
}

pub async fn begin_login(
    state: &AppState,
    email: String,
    user_agent: String,
    ip: IpAddr,
) -> Result<PasskeyChallengeResponse, AppError> {
    let normalized = email.trim().to_lowercase();
    let user = UserRepository::find_by_email(&state.pool, &normalized).await?;

    // Enumeration resistance: always return a challenge-like response, but only
    // bind a user id when the account exists. finish_login remains generic.
    let challenge_id = Uuid::new_v4().to_string();
    let challenge = Uuid::new_v4().to_string();
    let user_id = user.as_ref().map(|u| u.id);

    let stored = StoredPasskeyChallenge {
        user_id,
        email: Some(normalized),
        challenge: challenge.clone(),
        purpose: PasskeyChallengePurpose::Authentication,
        user_agent,
        ip: ip.to_string(),
    };

    state.redis.set_ex(
        &challenge_key(&challenge_id),
        &serde_json::to_string(&stored).map_err(|_| AppError::InternalError)?,
        CHALLENGE_TTL_SECONDS,
    ).await.map_err(|_| AppError::InternalError)?;

    let allow_credentials = match user_id {
        Some(id) => passkey_repository::list_user_passkeys(&state.pool, id)
            .await?
            .into_iter()
            .map(|credential| json!({
                "type": "public-key",
                "id": credential.credential_id,
                "transports": credential.transports
            }))
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    Ok(PasskeyChallengeResponse {
        challenge_id,
        public_key: json!({
            "challenge": challenge,
            "rpId": rp_id(),
            "allowCredentials": allow_credentials,
            "userVerification": "required",
            "timeout": 300000
        }),
    })
}

pub async fn finish_login(
    state: &AppState,
    challenge_id: &str,
    credential_id: &str,
    user_agent: String,
    ip: IpAddr,
) -> Result<LoginResult, AppError> {
    let stored = take_challenge(state, challenge_id).await?;

    if !matches!(stored.purpose, PasskeyChallengePurpose::Authentication) {
        return Err(AppError::Unauthorized);
    }

    let credential = passkey_repository::find_active_by_credential_id(&state.pool, credential_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if stored.user_id != Some(credential.user_id) {
        return Err(AppError::Unauthorized);
    }

    // HARDENING TODO before enabling in production:
    // Verify assertion signature over authenticatorData || SHA256(clientDataJSON),
    // compare challenge/origin/rpIdHash, require UV, reject cloned authenticators
    // when the sign counter regresses, and bind risk signals from IP/device.
    passkey_repository::update_successful_assertion(
        &state.pool,
        credential_id,
        credential.sign_count + 1,
    ).await?;

    let user = UserRepository::find_by_id(&state.pool, credential.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    auth_service::issue_full_login_tokens(
        &state.pool,
        &state.jwt_keys.encoding,
        &state.refresh_secret,
        &user,
        user_agent,
        ip,
    ).await
}

pub async fn list_passkeys(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<PasskeyCredentialView>, AppError> {
    let passkeys = passkey_repository::list_user_passkeys(&state.pool, user_id).await?;
    Ok(passkeys.into_iter().map(|credential| PasskeyCredentialView {
        credential_id: credential.credential_id,
        friendly_name: credential.friendly_name,
        transports: credential.transports,
        created_at: credential.created_at.to_rfc3339(),
        last_used_at: credential.last_used_at.map(|dt| dt.to_rfc3339()),
    }).collect())
}

pub async fn revoke_passkey(
    state: &AppState,
    user_id: i64,
    credential_id: &str,
) -> Result<(), AppError> {
    passkey_repository::revoke_passkey(&state.pool, user_id, credential_id).await
}

async fn take_challenge(state: &AppState, challenge_id: &str) -> Result<StoredPasskeyChallenge, AppError> {
    let key = challenge_key(challenge_id);
    let raw = state.redis.get_string(&key).await.map_err(|_| AppError::InternalError)?;
    state.redis.del(&key).await.map_err(|_| AppError::InternalError)?;

    let Some(raw) = raw else {
        return Err(AppError::Unauthorized);
    };

    serde_json::from_str(&raw).map_err(|_| AppError::Unauthorized)
}

fn challenge_key(challenge_id: &str) -> String {
    format!("passkey:challenge:{challenge_id}")
}

fn rp_id() -> String {
    std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string())
}

fn rp_name() -> String {
    std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "Zer0 Trust Auth".to_string())
}
