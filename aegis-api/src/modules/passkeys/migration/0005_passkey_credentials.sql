CREATE TABLE IF NOT EXISTS passkey_credentials (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id TEXT NOT NULL UNIQUE,
    public_key_cose BYTEA NOT NULL,
    sign_count BIGINT NOT NULL DEFAULT 0,
    friendly_name TEXT,
    transports TEXT[] NOT NULL DEFAULT '{}',
    aaguid TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_passkey_credentials_user_active
    ON passkey_credentials(user_id)
    WHERE revoked_at IS NULL;

COMMENT ON TABLE passkey_credentials IS
    'WebAuthn/passkey public-key credentials. Private keys never touch the server.';
