-- =====================================================================
-- MFA hardening: TOTP replay prevention
--
-- A TOTP code stays valid for its whole 30s step, and with skew=1 the
-- effective window is ~90s. Without recording which step was last spent,
-- an intercepted code can be replayed inside that window — which is
-- exactly the window an attacker on the wire has.
--
--   * user_mfa.last_used_step - highest TOTP step already spent by this
--                               user. A code is accepted only if its step
--                               is strictly greater.
--
-- The mfa_backup_codes table already exists (migration 0006); this
-- release finally ships the service that uses it.
--
-- Apply with: psql -d aegis -f 0008_mfa_replay_and_backup_codes.sql
-- =====================================================================

BEGIN;

ALTER TABLE public.user_mfa
    ADD COLUMN IF NOT EXISTS last_used_step bigint;

COMMENT ON COLUMN public.user_mfa.last_used_step IS
    'Highest TOTP step consumed; codes from this step or earlier are rejected as replays.';

COMMIT;
