-- =====================================================================
-- Email verification + MFA backup codes
--
-- Adds:
--   * users.email_verified_at   - timestamp when verified (NULL = unverified)
--   * email_verification_tokens - hashed, single-use, short-TTL tokens
--                                 (raw token sent out-of-band via email)
--   * mfa_backup_codes          - single-use recovery codes (hashed)
--
-- Apply with: psql -d testdb -f 0006_email_verification_and_backup_codes.sql
-- =====================================================================

BEGIN;

-- ---------- Email verification ----------
ALTER TABLE public.users
    ADD COLUMN IF NOT EXISTS email_verified_at timestamptz;

CREATE TABLE IF NOT EXISTS public.email_verification_tokens (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    bigint NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    token_hash text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    used_at    timestamptz
);

CREATE INDEX IF NOT EXISTS idx_evt_token_hash ON public.email_verification_tokens (token_hash);
CREATE INDEX IF NOT EXISTS idx_evt_user_id    ON public.email_verification_tokens (user_id);

-- ---------- MFA backup codes (table prepared; service ships in a follow-up) ----------
CREATE TABLE IF NOT EXISTS public.mfa_backup_codes (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    bigint NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    code_hash  text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    used_at    timestamptz
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_mfa_backup_user_code
    ON public.mfa_backup_codes (user_id, code_hash);
CREATE INDEX IF NOT EXISTS idx_mfa_backup_user_id ON public.mfa_backup_codes (user_id);

COMMIT;
