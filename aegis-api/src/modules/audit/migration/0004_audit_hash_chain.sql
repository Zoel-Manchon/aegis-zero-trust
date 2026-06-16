-- =====================================================================
-- Tamper-evident audit log: hash chaining for security_events
--
-- Each event stores:
--   seq         : monotonically increasing position in the chain
--   prev_hash   : the event_hash of the previous event (genesis for the first)
--   event_hash  : SHA-256 over (prev_hash || canonical event fields)
--
-- Any modification to a row changes its event_hash, which no longer matches
-- the prev_hash recorded by the NEXT event — so tampering anywhere breaks the
-- chain from that point on and is detectable by re-walking it.
--
-- Concurrency: inserts are serialized via a Postgres advisory lock in the
-- repository so the chain stays strictly linear (no two events sharing a seq
-- or prev_hash).
--
-- Apply:  psql -d testdb -f 0004_audit_hash_chain.sql
-- =====================================================================

BEGIN;

ALTER TABLE public.security_events
    ADD COLUMN IF NOT EXISTS seq        bigint,
    ADD COLUMN IF NOT EXISTS prev_hash  text,
    ADD COLUMN IF NOT EXISTS event_hash text;

-- Fast ordered walk of the chain.
CREATE INDEX IF NOT EXISTS idx_security_events_seq ON public.security_events (seq);

-- Each hash is unique once populated (a duplicate would indicate a problem).
CREATE UNIQUE INDEX IF NOT EXISTS uq_security_events_event_hash
    ON public.security_events (event_hash)
    WHERE event_hash IS NOT NULL;

COMMIT;
