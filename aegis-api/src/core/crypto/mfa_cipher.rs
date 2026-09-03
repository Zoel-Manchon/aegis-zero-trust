// =============================================================================
// Envelope encryption for TOTP seeds at rest.
//
// THE PROBLEM
//   `user_mfa.secret` is the TOTP seed. Whoever reads that column can mint
//   valid codes for that account, forever, without touching the app: a database
//   dump, a nightly backup, a read replica or one SQL injection is enough. The
//   password is Argon2-hashed and the backup codes are Argon2-hashed, so today
//   the seed is the softest thing in the table — and it is the one secret that
//   *cannot* be hashed, because the server has to recompute the code.
//
// THE SHAPE OF THE FIX
//   Envelope encryption. Every seed gets its own random 256-bit data key (DEK).
//   The seed is sealed with AES-256-GCM under that DEK; the DEK is then wrapped
//   by a key-encryption key (KEK) that never lives in the database:
//
//       seed --AES-256-GCM(DEK)--> ciphertext
//       DEK  --wrap(KEK)---------> wrapped DEK
//       stored: aegis.v1.<wrapped DEK>.<nonce>.<ciphertext>
//
//   With Vault as the KEK holder, the wrap is a `transit/encrypt` call, so the
//   key material stays inside Vault and this process only ever handles a
//   32-byte DEK it just generated. Rotating the transit key re-wraps future
//   rows without touching a single seed; revoking it makes every stored seed
//   unreadable, which is exactly what you want on the day you need it.
//
// WHY NOT ENCRYPT THE SEED WITH THE KEK DIRECTLY
//   Because then every row is encrypted under one key, key rotation means
//   decrypting and rewriting the whole table, and Vault would see the seeds
//   themselves. With an envelope, Vault sees only random 32-byte DEKs.
//
// CONFIGURATION
//   MFA_KEY_WRAPPER=vault   wrap DEKs with Vault transit. Needs VAULT_ADDR and
//                           VAULT_TOKEN; MFA_TRANSIT_KEY defaults to aegis-mfa.
//   MFA_KEK=<base64 32B>    wrap DEKs locally with AES-256-GCM. For development
//                           and CI, where standing up Vault is not worth it.
//   neither                 disabled: seeds are stored exactly as before, in
//                           plaintext, and startup says so out loud.
//
// READING OLD ROWS
//   A stored value that does not carry the `aegis.v1.` tag is a legacy
//   plaintext seed and is returned as-is. Enrolments written from now on are
//   sealed, so the table converts as users re-enrol; `is_legacy_plaintext` is
//   what an operator-facing migration job would use to force the issue.
// =============================================================================

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use rand::{RngCore, rngs::OsRng};

use crate::core::errors::app_error::AppError;

/// Marks a stored value as an envelope. Anything without it is legacy plaintext.
const ENVELOPE_TAG: &str = "aegis.v1";

/// AES-GCM nonce length in bytes (96 bits, the only size the spec blesses).
const NONCE_LEN: usize = 12;

/// Data-encryption key length in bytes.
const DEK_LEN: usize = 32;

/// Prefix on a locally wrapped DEK, so the two wrapper kinds can never be
/// mistaken for one another when a deployment switches from one to the other.
const LOCAL_WRAP_PREFIX: &str = "local:v1:";

/// How the DEK gets wrapped. The seed itself never reaches any of these.
enum KeyWrapper {
    /// Vault's transit engine holds the KEK; we send it a DEK and store the
    /// `vault:v1:...` blob it hands back.
    VaultTransit {
        address: String,
        token: String,
        key_name: String,
        http: reqwest::Client,
    },
    /// AES-256-GCM under a KEK from the environment. Same guarantee against a
    /// database-only compromise; no protection if the process env leaks too.
    LocalKek { kek: Vec<u8> },
    /// No KEK configured. Seeds are stored in plaintext, as they were before
    /// this module existed.
    Disabled,
}

pub struct MfaCipher {
    wrapper: KeyWrapper,
}

impl MfaCipher {
    /// Build from the environment. Never fails: a misconfigured wrapper falls
    /// back to `Disabled` with a loud warning, because refusing to boot would
    /// take down authentication for everyone to protect a column that was
    /// already plaintext a minute ago.
    pub fn from_env() -> Self {
        let mode = std::env::var("MFA_KEY_WRAPPER").unwrap_or_default();

        if mode.eq_ignore_ascii_case("vault") {
            match (std::env::var("VAULT_ADDR"), std::env::var("VAULT_TOKEN")) {
                (Ok(address), Ok(token)) if !address.is_empty() && !token.is_empty() => {
                    let key_name = std::env::var("MFA_TRANSIT_KEY")
                        .unwrap_or_else(|_| "aegis-mfa".to_string());
                    tracing::info!(
                        key_name,
                        "MFA seeds: envelope encryption with Vault transit"
                    );
                    return Self {
                        wrapper: KeyWrapper::VaultTransit {
                            address: address.trim_end_matches('/').to_string(),
                            token,
                            key_name,
                            http: reqwest::Client::new(),
                        },
                    };
                }
                _ => {
                    tracing::error!(
                        "MFA_KEY_WRAPPER=vault but VAULT_ADDR/VAULT_TOKEN are missing; \
                         TOTP seeds will be stored in PLAINTEXT"
                    );
                    return Self { wrapper: KeyWrapper::Disabled };
                }
            }
        }

        match std::env::var("MFA_KEK") {
            Ok(encoded) if !encoded.is_empty() => match decode_kek(&encoded) {
                Ok(kek) => {
                    tracing::info!("MFA seeds: envelope encryption with a local KEK");
                    Self { wrapper: KeyWrapper::LocalKek { kek } }
                }
                Err(reason) => {
                    tracing::error!(
                        reason,
                        "MFA_KEK is not a base64-encoded 32-byte key; \
                         TOTP seeds will be stored in PLAINTEXT"
                    );
                    Self { wrapper: KeyWrapper::Disabled }
                }
            },
            _ => {
                tracing::warn!(
                    "No MFA_KEK and no Vault transit configured: TOTP seeds are stored in \
                     PLAINTEXT. Set MFA_KEK (base64, 32 bytes) or MFA_KEY_WRAPPER=vault."
                );
                Self { wrapper: KeyWrapper::Disabled }
            }
        }
    }

    /// Test/embedding constructor for a known local KEK.
    pub fn with_local_kek(kek: Vec<u8>) -> Self {
        Self { wrapper: KeyWrapper::LocalKek { kek } }
    }

    /// A cipher that stores plaintext, for tests that care about other things.
    pub fn disabled() -> Self {
        Self { wrapper: KeyWrapper::Disabled }
    }

    /// True when seeds actually get sealed. Surfaced so a health endpoint or a
    /// startup banner can state the posture instead of implying it.
    pub fn is_active(&self) -> bool {
        !matches!(self.wrapper, KeyWrapper::Disabled)
    }

    /// Seal a seed for storage. With no KEK configured this returns the seed
    /// unchanged, which is the pre-existing behaviour and is tagged as such by
    /// its lack of an envelope prefix.
    pub async fn seal(&self, plaintext: &str) -> Result<String, AppError> {
        if let KeyWrapper::Disabled = self.wrapper {
            return Ok(plaintext.to_string());
        }

        let mut dek = [0u8; DEK_LEN];
        OsRng.fill_bytes(&mut dek);

        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);

        let cipher =
            Aes256Gcm::new_from_slice(&dek).map_err(|_| AppError::InternalError)?;
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_bytes())
            .map_err(|_| AppError::InternalError)?;

        let wrapped = self.wrap_dek(&dek).await?;

        Ok(format!(
            "{ENVELOPE_TAG}.{}.{}.{}",
            B64.encode(wrapped.as_bytes()),
            B64.encode(nonce_bytes),
            B64.encode(ciphertext)
        ))
    }

    /// Recover a seed. Legacy plaintext passes through untouched, so a table
    /// mid-migration keeps working for everyone in it.
    pub async fn open(&self, stored: &str) -> Result<String, AppError> {
        if !is_envelope(stored) {
            return Ok(stored.to_string());
        }

        // "aegis" . "v1" . wrapped . nonce . ciphertext — the tag carries its
        // own dot, so the split is five fields, not three.
        let fields: Vec<&str> = stored.splitn(5, '.').collect();
        let [_, _, wrapped_b64, nonce_b64, ciphertext_b64] = fields[..] else {
            tracing::error!("stored MFA secret is tagged as an envelope but malformed");
            return Err(AppError::InternalError);
        };

        let wrapped_bytes = B64.decode(wrapped_b64).map_err(|_| AppError::InternalError)?;
        let wrapped = String::from_utf8(wrapped_bytes).map_err(|_| AppError::InternalError)?;
        let nonce_bytes = B64.decode(nonce_b64).map_err(|_| AppError::InternalError)?;
        let ciphertext = B64.decode(ciphertext_b64).map_err(|_| AppError::InternalError)?;

        if nonce_bytes.len() != NONCE_LEN {
            return Err(AppError::InternalError);
        }

        let dek = self.unwrap_dek(&wrapped).await?;

        let cipher = Aes256Gcm::new_from_slice(&dek).map_err(|_| AppError::InternalError)?;
        let plaintext = cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
            .map_err(|_| {
                // GCM failing here means the ciphertext or the DEK was tampered
                // with. Never fall back to treating the blob as plaintext.
                tracing::error!("MFA secret failed authenticated decryption");
                AppError::InternalError
            })?;

        String::from_utf8(plaintext).map_err(|_| AppError::InternalError)
    }

    async fn wrap_dek(&self, dek: &[u8]) -> Result<String, AppError> {
        match &self.wrapper {
            KeyWrapper::Disabled => Err(AppError::InternalError),

            KeyWrapper::LocalKek { kek } => {
                let mut nonce_bytes = [0u8; NONCE_LEN];
                OsRng.fill_bytes(&mut nonce_bytes);

                let cipher =
                    Aes256Gcm::new_from_slice(kek).map_err(|_| AppError::InternalError)?;
                let wrapped = cipher
                    .encrypt(Nonce::from_slice(&nonce_bytes), dek)
                    .map_err(|_| AppError::InternalError)?;

                let mut blob = nonce_bytes.to_vec();
                blob.extend_from_slice(&wrapped);

                Ok(format!("{LOCAL_WRAP_PREFIX}{}", B64.encode(blob)))
            }

            KeyWrapper::VaultTransit { address, token, key_name, http } => {
                let url = format!("{address}/v1/transit/encrypt/{key_name}");
                let response = http
                    .post(&url)
                    .header("X-Vault-Token", token)
                    .json(&serde_json::json!({
                        "plaintext": base64::engine::general_purpose::STANDARD.encode(dek)
                    }))
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::error!(error = %e, "Vault transit encrypt failed");
                        AppError::InternalError
                    })?;

                let body: serde_json::Value =
                    response.json().await.map_err(|_| AppError::InternalError)?;

                body["data"]["ciphertext"]
                    .as_str()
                    .map(str::to_string)
                    .ok_or(AppError::InternalError)
            }
        }
    }

    async fn unwrap_dek(&self, wrapped: &str) -> Result<Vec<u8>, AppError> {
        match &self.wrapper {
            KeyWrapper::Disabled => Err(AppError::InternalError),

            KeyWrapper::LocalKek { kek } => {
                let blob = wrapped
                    .strip_prefix(LOCAL_WRAP_PREFIX)
                    .ok_or(AppError::InternalError)
                    .and_then(|b| B64.decode(b).map_err(|_| AppError::InternalError))?;

                if blob.len() <= NONCE_LEN {
                    return Err(AppError::InternalError);
                }

                let (nonce_bytes, sealed) = blob.split_at(NONCE_LEN);
                let cipher =
                    Aes256Gcm::new_from_slice(kek).map_err(|_| AppError::InternalError)?;

                cipher
                    .decrypt(Nonce::from_slice(nonce_bytes), sealed)
                    .map_err(|_| AppError::InternalError)
            }

            KeyWrapper::VaultTransit { address, token, key_name, http } => {
                let url = format!("{address}/v1/transit/decrypt/{key_name}");
                let response = http
                    .post(&url)
                    .header("X-Vault-Token", token)
                    .json(&serde_json::json!({ "ciphertext": wrapped }))
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::error!(error = %e, "Vault transit decrypt failed");
                        AppError::InternalError
                    })?;

                let body: serde_json::Value =
                    response.json().await.map_err(|_| AppError::InternalError)?;

                let encoded = body["data"]["plaintext"]
                    .as_str()
                    .ok_or(AppError::InternalError)?;

                base64::engine::general_purpose::STANDARD
                    .decode(encoded)
                    .map_err(|_| AppError::InternalError)
            }
        }
    }
}

/// Does this stored value carry an envelope, or is it a legacy plaintext seed?
pub fn is_envelope(stored: &str) -> bool {
    stored.starts_with(ENVELOPE_TAG)
}

/// The inverse, named for the call site that matters: a migration job asking
/// "does this row still need sealing?".
pub fn is_legacy_plaintext(stored: &str) -> bool {
    !is_envelope(stored)
}

fn decode_kek(encoded: &str) -> Result<Vec<u8>, &'static str> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .map_err(|_| "not valid base64")?;

    if bytes.len() != DEK_LEN {
        return Err("wrong length: a KEK must be exactly 32 bytes");
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cipher() -> MfaCipher {
        MfaCipher::with_local_kek(vec![7u8; DEK_LEN])
    }

    #[tokio::test]
    async fn a_sealed_seed_round_trips() {
        let c = cipher();
        let seed = "JBSWY3DPEHPK3PXP";

        let stored = c.seal(seed).await.expect("seal");
        assert_eq!(c.open(&stored).await.expect("open"), seed);
    }

    #[tokio::test]
    async fn the_stored_form_does_not_contain_the_seed() {
        let seed = "JBSWY3DPEHPK3PXP";
        let stored = cipher().seal(seed).await.expect("seal");

        assert!(is_envelope(&stored));
        assert!(!stored.contains(seed), "the seed leaked into the stored value");
    }

    #[tokio::test]
    async fn every_seal_uses_a_fresh_data_key_and_nonce() {
        let c = cipher();
        let a = c.seal("JBSWY3DPEHPK3PXP").await.expect("seal");
        let b = c.seal("JBSWY3DPEHPK3PXP").await.expect("seal");

        // Same seed, same KEK, different stored value: no deterministic
        // ciphertext to correlate two accounts that enrolled the same secret.
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn tampering_with_the_ciphertext_is_rejected() {
        let c = cipher();
        let stored = c.seal("JBSWY3DPEHPK3PXP").await.expect("seal");

        let mut fields: Vec<String> = stored.split('.').map(str::to_string).collect();
        let last = fields.len() - 1;
        // Flip one character of the ciphertext.
        let ct = &fields[last];
        let flipped = if ct.starts_with('A') { format!("B{}", &ct[1..]) } else { format!("A{}", &ct[1..]) };
        fields[last] = flipped;

        assert!(c.open(&fields.join(".")).await.is_err());
    }

    #[tokio::test]
    async fn another_kek_cannot_open_the_envelope() {
        let stored = cipher().seal("JBSWY3DPEHPK3PXP").await.expect("seal");
        let stranger = MfaCipher::with_local_kek(vec![9u8; DEK_LEN]);

        assert!(stranger.open(&stored).await.is_err());
    }

    #[tokio::test]
    async fn legacy_plaintext_rows_still_read() {
        // A row written before this module existed: no tag, no envelope.
        let legacy = "JBSWY3DPEHPK3PXP";

        assert!(is_legacy_plaintext(legacy));
        assert_eq!(cipher().open(legacy).await.expect("open"), legacy);
    }

    #[tokio::test]
    async fn a_disabled_cipher_is_a_no_op_in_both_directions() {
        let c = MfaCipher::disabled();
        let seed = "JBSWY3DPEHPK3PXP";

        assert!(!c.is_active());
        assert_eq!(c.seal(seed).await.expect("seal"), seed);
        assert_eq!(c.open(seed).await.expect("open"), seed);
    }

    #[test]
    fn a_kek_must_be_exactly_thirty_two_bytes() {
        use base64::engine::general_purpose::STANDARD;

        assert!(decode_kek(&STANDARD.encode([0u8; 32])).is_ok());
        assert!(decode_kek(&STANDARD.encode([0u8; 16])).is_err());
        assert!(decode_kek("not base64!").is_err());
    }

    #[test]
    fn a_malformed_envelope_is_never_mistaken_for_plaintext() {
        assert!(is_envelope("aegis.v1.broken"));
        assert!(!is_legacy_plaintext("aegis.v1.broken"));
    }

    #[tokio::test]
    async fn a_malformed_envelope_fails_loudly_instead_of_leaking() {
        assert!(cipher().open("aegis.v1.broken").await.is_err());
    }
}
