# DB init

Postgres runs every `*.sql` here **once**, on first init (empty data dir),
alphabetically. Drop your authoritative schema in as `01_schema.sql`:

```bash
# from a machine with your WORKING database:
pg_dump --schema-only --no-owner --no-privileges "$DATABASE_URL" \
  > docker/db-init/01_schema.sql
```

That dump already includes every table, the hash-chained audit setup, and
(if you applied it) the SOC NOTIFY trigger. `99_soc_notify.sql` re-applies the
trigger idempotently as a safety net.

To re-seed the schema after changes: `docker compose down -v` then `up` again.
