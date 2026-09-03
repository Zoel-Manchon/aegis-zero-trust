-- =====================================================================
-- MFA hardening: TOTP seeds encrypted at rest
--
-- `user_mfa.secret` used to hold the raw base32 TOTP seed. Anyone who
-- could read the row could mint valid codes for that account forever —
-- a dump, a backup, a read replica or one SQL injection was enough.
-- The seed cannot be hashed (the server has to recompute the code), so
-- it is now encrypted instead.
--
-- The column keeps its type. What changes is the *format* of the value:
--
--   legacy   JBSWY3DPEHPK3PXP...          raw base32, still readable
--   sealed   aegis.v1.<dek>.<nonce>.<ct>  AES-256-GCM under a per-seed
--                                         data key, that key wrapped by
--                                         Vault transit or a local KEK
--
-- Both forms are accepted on read (core::crypto::mfa_cipher), so no
-- downtime and no lockout: rows convert to the sealed form as users
-- enrol or re-enrol. There is deliberately no in-database migration —
-- sealing requires the KEK, which lives outside the database, which is
-- the entire point.
--
-- Find the rows still in the clear:
--
--   SELECT user_id, created_at
--   FROM public.user_mfa
--   WHERE secret NOT LIKE 'aegis.v1.%'
--   ORDER BY created_at;
--
-- Apply with: psql -d aegis -f 0009_mfa_secret_envelope.sql
-- =====================================================================

BEGIN;

COMMENT ON COLUMN public.user_mfa.secret IS
    'TOTP seed. Sealed rows carry the aegis.v1.<wrapped DEK>.<nonce>.<ciphertext> envelope (AES-256-GCM, key wrapped by Vault transit or a local KEK). Rows without that prefix are legacy plaintext seeds and are read as-is until re-enrolment.';

COMMIT;
