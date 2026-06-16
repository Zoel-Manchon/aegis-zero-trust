-- ============================================================================
-- 0007_soc_event_notify.sql
--
-- Real-time SOC push pipeline (publish side).
--
-- Every insert into security_events emits a Postgres NOTIFY on the 'soc_events'
-- channel with the full row as JSON. The admin SSE endpoint
-- (/admin/security/events/stream) holds a LISTEN connection and relays each
-- notification to connected dashboards instantly — no polling.
--
-- This is the publish side; it touches NO application code, so the
-- tamper-evident audit-chain insert path is unchanged.
--
-- NOTE: pg_notify payloads are capped at 8000 bytes. security_events rows are
-- well under that; if you ever store large metadata blobs, switch the payload
-- to a compact summary (id, event_type, severity, created_at) and have the SSE
-- handler re-fetch the full row by id.
-- ============================================================================

CREATE OR REPLACE FUNCTION soc_notify_security_event() RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('soc_events', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_soc_notify_security_event ON security_events;

CREATE TRIGGER trg_soc_notify_security_event
    AFTER INSERT ON security_events
    FOR EACH ROW
    EXECUTE FUNCTION soc_notify_security_event();
