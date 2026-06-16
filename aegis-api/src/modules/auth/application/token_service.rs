use crate::core::crypto::jwt;
use crate::modules::auth::models::user_model::User;
use base64::{Engine as _, engine::general_purpose};
use hmac::KeyInit;
use hmac::{Hmac, Mac};
use jsonwebtoken::EncodingKey;
use rand::{RngCore, rngs::OsRng};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const REFRESH_TOKEN_DOMAIN: &[u8] = b"refresh-token-v1:";

// ---------------------------------------------------------------------------
// REFRESH TOKEN HASHING
// ---------------------------------------------------------------------------

pub fn hash_refresh_token(
    token: &str,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| {
        jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)
    })?;

    mac.update(REFRESH_TOKEN_DOMAIN);
    mac.update(token.as_bytes());

    Ok(hex::encode(mac.finalize().into_bytes()))
}

// ---------------------------------------------------------------------------
// TOKEN PAIR
// ---------------------------------------------------------------------------

pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_token_hash: String,
    pub jti: Uuid,
}

// ---------------------------------------------------------------------------
// GENERATION
// ---------------------------------------------------------------------------

pub fn generate_token_pair(
    user: &User,
    encoding_key: &EncodingKey,
    refresh_secret: &str,
) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    let jti = Uuid::new_v4();

    let access_token = jwt::generate_token(encoding_key, user.id, &jti)?;

    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    let refresh_token = general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let refresh_token_hash = hash_refresh_token(&refresh_token, refresh_secret)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        refresh_token_hash,
        jti,
    })
}
