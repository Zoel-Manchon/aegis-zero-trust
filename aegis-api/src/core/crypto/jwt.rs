use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub jti: String,
    pub purpose: String,
    pub iat: usize,
    pub nbf: usize,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
}

fn jwt_issuer() -> String {
    std::env::var("JWT_ISSUER").unwrap_or_else(|_| "auth-service".to_string())
}

fn jwt_audience() -> String {
    std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "auth-service-users".to_string())
}

pub fn generate_token(
    encoding_key: &EncodingKey,
    user_id: i64,
    jti: &Uuid,
) -> Result<String, jsonwebtoken::errors::Error> {
    generate_token_with_purpose(
        encoding_key,
        user_id,
        jti,
        "access",
        access_token_ttl_minutes(),
    )
}

pub fn generate_mfa_token(
    encoding_key: &EncodingKey,
    user_id: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let jti = Uuid::new_v4();

    generate_token_with_purpose(encoding_key, user_id, &jti, "mfa", 5)
}

pub fn verify_token(
    token: &str,
    decoding_key: &DecodingKey,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let claims = decode_claims(token, decoding_key)?;

    if claims.purpose != "access" {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }

    Ok(claims)
}

pub fn verify_mfa_token(
    token: &str,
    decoding_key: &DecodingKey,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let claims = decode_claims(token, decoding_key)?;

    if claims.purpose != "mfa" {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }

    Ok(claims)
}

fn generate_token_with_purpose(
    encoding_key: &EncodingKey,
    user_id: i64,
    jti: &Uuid,
    purpose: &str,
    ttl_minutes: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let now_ts = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        jti: jti.to_string(),
        purpose: purpose.to_string(),
        iat: now_ts,
        nbf: now_ts,
        exp: (now + Duration::minutes(ttl_minutes)).timestamp() as usize,
        iss: jwt_issuer(),
        aud: jwt_audience(),
    };

    let mut header = Header::new(Algorithm::RS256);
    header.typ = Some("JWT".to_string());

    encode(&header, &claims, encoding_key)
}

fn decode_claims(
    token: &str,
    decoding_key: &DecodingKey,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::RS256);

    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_audience(&[jwt_audience()]);
    validation.set_issuer(&[jwt_issuer()]);
    validation.leeway = 5;

    decode::<Claims>(token, decoding_key, &validation).map(|data| data.claims)
}

fn access_token_ttl_minutes() -> i64 {
    std::env::var("ACCESS_TOKEN_TTL_MINUTES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(15)
}
