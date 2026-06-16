-- Real-time SOC NOTIFY trigger (safety net; the schema dump may already have it).
-- Guarded: if security_events isn't present yet, skip cleanly instead of
-- crashing initdb — so a missing 01_schema.sql gives a clear message, not exit 3.
DO $do$
BEGIN
    IF to_regclass('public.security_events') IS NULL THEN
        RAISE NOTICE 'security_events not found — skipping SOC trigger. Did you add 01_schema.sql?';
        RETURN;
    END IF;

    CREATE OR REPLACE FUNCTION soc_notify_security_event() RETURNS trigger
    LANGUAGE plpgsql AS $fn$
    BEGIN
        PERFORM pg_notify('soc_events', row_to_json(NEW)::text);
        RETURN NEW;
    END;
    $fn$;

    DROP TRIGGER IF EXISTS trg_soc_notify_security_event ON security_events;
    CREATE TRIGGER trg_soc_notify_security_event
        AFTER INSERT ON security_events
        FOR EACH ROW EXECUTE FUNCTION soc_notify_security_event();
END
$do$;
