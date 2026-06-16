# Running aegis with Docker

One command brings up Postgres, Redis, the Rust API, and the console behind
Caddy on a single origin. Two one-time prerequisites first.

## Two generated files (committed in this repo)

A fresh clone runs as-is because these are committed. You only regenerate them if
you **change the schema** or the **`query!` macros**:

**1. Schema — `docker/db-init/01_schema.sql`.** There's no auto-migrator; the DB
container loads this on first boot. Regenerate from a working database:

```bash
PGPASSWORD=... pg_dump -h localhost -p 5432 -U <user> -d <db> \
  --schema-only --no-owner --no-privileges > docker/db-init/01_schema.sql
grep -c "CREATE TABLE" docker/db-init/01_schema.sql   # sanity: should be > 0
```

> ⚠️ **Match the Postgres major version.** `pg_dump` emits settings specific to
> its server version (e.g. `transaction_timeout` exists only in PG 17+). The DB
> container is pinned to **postgres:17** in `docker-compose.yml`; if your dump
> comes from a different major version, change that tag to match.

**2. sqlx offline cache — `aegis-api/.sqlx/`.** The API uses compile-time `sqlx`
macros, so the image builds offline against this cache. Regenerate:

```bash
cd aegis-api
# the CLI MUST be built with the postgres driver, or you'll get "no driver found":
cargo install sqlx-cli --force --no-default-features --features rustls,postgres
# use the postgres:// scheme (NOT postgresql://):
DATABASE_URL="postgres://<user>:<pass>@localhost:5432/<db>" cargo sqlx prepare
```

## Up

```bash
docker compose up --build          # db + redis + api + web
```

Open **http://localhost:8080**.

Seed the demo accounts (separate one-shot, after the stack is up):

```bash
docker compose --profile seed run --rm seed
#   admin : admin@test.com  / AdminPass123!   (enroll MFA on first login)
#   victim: victim@test.com / VictimPass123!
```

## Use it

1. Sign in as `admin@test.com` → you'll hit the **MFA gate** (mandatory for
   admins) → enroll TOTP on the account page → sign in again.
2. In the SOC, open **Attack Range**, pick an origin + scenario, target
   `victim@test.com`, and launch. Launch again from a distant origin to trip
   **IMPOSSIBLE_TRAVEL** — watch the live feed, map, and the WS popup + chime.

## How it's wired

- **Single origin.** Caddy serves the console and proxies `/api/*` → `api:3000`
  (stripping `/api`). SSE (events) and WS (alerts) upgrade through Caddy
  automatically, and the backend sees the real client IP via `X-Forwarded-For`.
- **JWT keys** are generated on first API boot into the `api-keys` volume.
- **DB TLS** is off on the internal network (`DB_SSL_MODE=disable`). For real
  TLS, set it to `require`/`verify-full` and give Postgres a cert.
- **HTTPS:** see the commented `https://localhost { tls internal }` block in
  `docker/Caddyfile` for single-origin TLS (reference parity).

## Troubleshooting

The DB only runs `docker/db-init/*.sql` on a **fresh** data volume. After any
schema change, wipe and recreate: `docker compose down -v && docker compose up -d --build`.

| Symptom | Cause / fix |
| --- | --- |
| Build fails on a `query!` macro (`cargo build` exit 101) | `.sqlx` cache missing — run the `cargo sqlx prepare` step above. |
| `cargo sqlx prepare`: *no driver found for URL scheme "postgres"* | sqlx-cli built without the Postgres driver — reinstall with `--features rustls,postgres --force`. |
| `cargo sqlx prepare`: *no driver found for ... "postgresql"* | Use the `postgres://` scheme, not `postgresql://`. |
| DB init: *unrecognized configuration parameter "transaction_timeout"* | Dump is from a newer Postgres than the container — match the `postgres:NN` tag (see version note above). |
| Runtime / seed: *relation "users" (or other) does not exist* | `01_schema.sql` is missing/empty, so the DB initialized with no tables. Add it, then `docker compose down -v && up -d --build`. |
| `seed` fails: *network ... not found* | Stale profiled container from a previous run. Use `docker compose --profile seed run --rm seed` (one-off), not `up seed`. |
| Port 8080 already in use | Another stack (e.g. an old project) is bound to it — tear it down, or change `8080:80` under the `web` service. |

Verify the schema actually loaded before seeding:
`docker compose exec db psql -U aegis -d aegis -c '\dt'` (should list `users`,
`sessions`, `security_events`, …).

## Notes

- Change `REFRESH_SECRET` and the DB password in `docker-compose.yml` before
  anything beyond local use.
- The backend's correctness depends on `01_schema.sql` matching the columns the
  code expects; it's committed so clones are reproducible.
