-- =====================================================================
-- Password reset tokens
--
-- Zero-trust design choices:
--   * We store ONLY a SHA-256 hash of the token, never the raw value. A DB
--     leak therefore does not hand an attacker usable reset links.
--   * Tokens are single-use: `used_at` is set the moment a reset succeeds, and
--     the lookup rejects already-used tokens.
--   * Tokens are short-lived: `expires_at` (the service sets ~30 min).
--   * ON DELETE CASCADE so removing a user cleans up their tokens.
--
-- Apply:  psql -d testdb -f 0002_password_reset_tokens.sql
-- =====================================================================

BEGIN;

CREATE TABLE IF NOT EXISTS public.password_reset_tokens (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     bigint NOT NULL
                    REFERENCES public.users(id) ON DELETE CASCADE,
    -- SHA-256 hex digest of the raw token (64 chars). Never the raw token.
    token_hash  text   NOT NULL UNIQUE,
    created_at  timestamptz NOT NULL DEFAULT now(),
    expires_at  timestamptz NOT NULL,
    used_at     timestamptz
);

-- Fast lookup by hash on the reset path.
CREATE INDEX IF NOT EXISTS idx_prt_token_hash ON public.password_reset_tokens (token_hash);
-- Lets us invalidate / inspect all outstanding tokens for a user.
CREATE INDEX IF NOT EXISTS idx_prt_user_id    ON public.password_reset_tokens (user_id);

COMMIT;
