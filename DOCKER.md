# Running aegis with Docker

One command brings up Postgres, Redis, the Rust API and the console behind Caddy
on a single origin.

Requirements: **Docker Engine 24+ / Docker Desktop** (Compose v2 + BuildKit, both
default). No local Rust or Node toolchain needed.

## Up

```bash
git clone https://github.com/Zoel-Manchon/aegis-zero-trust.git
cd aegis-zero-trust

docker compose up -d --build                   # db + redis + api + web
docker compose --profile seed run --rm seed    # demo accounts
```

Open **https://localhost**.

```
admin : admin@test.com  / AdminPass123!    (must enroll TOTP on first login)
victim: victim@test.com / VictimPass123!
```

The first build takes ~5-10 min (Rust release build). Later builds reuse the
cargo cache mount and take seconds.

```bash
docker compose ps                     # health of every service
docker compose logs -f api            # backend logs
docker compose down                   # stop, keep data
docker compose down -v                # stop and WIPE the database volume
```

## With Vault (dynamic DB credentials)

The API can take **short-lived, per-instance Postgres credentials** from
HashiCorp Vault instead of the static password:

```bash
docker compose -f docker-compose.yml -f docker-compose.vault.yml up -d --build
docker compose --profile seed run --rm seed
# console -> https://localhost     Vault UI -> http://localhost:8200 (token: root)
```

`vault-init` (`vault/init.sh`) enables the database secrets engine and defines
the `aegis-api` role; the API entrypoint requests `database/creds/aegis-api` on
boot and builds `DATABASE_URL` from the leased username/password. Inspect it:

```bash
docker compose exec vault sh -c 'VAULT_ADDR=http://127.0.0.1:8200 VAULT_TOKEN=root vault read database/creds/aegis-api'
docker compose exec db psql -U aegis -d aegis -c '\du' | grep v-token
```

Dev-mode Vault is in-memory with a known root token: a lab demonstration, not a
production posture. Credentials are fetched once at container start (1h lease);
a real deployment would renew the lease in-process.

## Two generated files (already committed)

A fresh clone runs as-is. Regenerate these only if you change the schema or a
`query!` macro.

**1. Schema - `docker/db-init/01_schema.sql`.** There is no auto-migrator; the DB
container loads this on first boot.

```bash
PGPASSWORD=... pg_dump -h localhost -p 5432 -U <user> -d <db> \
  --schema-only --no-owner --no-privileges > docker/db-init/01_schema.sql
```

> Match the Postgres major version. `pg_dump` emits version-specific settings
> (`transaction_timeout` is PG 17+). The container is pinned to `postgres:17`.

**2. sqlx offline cache - `aegis-api/.sqlx/`.** The API uses compile-time `query!`
macros, so the image builds offline against this cache. **Any edit to the SQL
text inside a `query!` macro invalidates its entry** and the build fails with
`SQLX_OFFLINE=true but there is no cached data for this query`. Regenerate:

```bash
# expose the DB on the host first: uncomment the `ports:` block under `db`
docker compose up -d db

cd aegis-api
cargo install sqlx-cli --force --no-default-features --features rustls,postgres
DATABASE_URL="postgres://aegis:aegis@localhost:5433/aegis" cargo sqlx prepare
```

`cargo sqlx prepare` rewrites `.sqlx/` from scratch: it adds entries for new
queries and deletes orphaned ones. Commit the result. Sanity check - the number
of files must equal the number of macro call sites:

```bash
ls aegis-api/.sqlx/*.json | wc -l
grep -rohE 'sqlx::query(_as|_scalar)?!' aegis-api/src | wc -l
```

## How it's wired

- **Single origin.** Caddy serves the console and proxies `/api/*` -> `api:3000`
  (stripping `/api`). SSE and WebSocket upgrade through automatically, and the
  backend sees the real client IP via `X-Forwarded-For` (used by GeoIP).
- **JWT keys** are generated on first API boot into the `api-keys` volume.
- **Health gating.** `web` waits for `api` to report healthy, `api` waits for
  Postgres and Redis, so a cold `up` never serves a console that 502s.
- **Build context.** `web` builds from the repo root, so the root
  `.dockerignore` is what keeps `target/` and `node_modules/` out of it.
- **Line endings.** `.gitattributes` forces LF on everything executed inside a
  container. A CRLF shebang makes the kernel look for `/bin/sh\r`.
- **HTTPS:** see the commented `https://localhost { tls internal }` block in
  `docker/Caddyfile`.

## Extra binaries in the API image

The image ships three binaries. `aegis` is the server (the entrypoint); the
other two are red-team tools you can run against the live stack:

```bash
docker compose exec api attack_simulator                          # one-shot pentest battery, PASS/FAIL
docker compose exec api attack_simulator storm --rps 20 --secs 15 # continuous multi-vector load
docker compose exec -it api admin_terminal                        # interactive alert generator
```

## Troubleshooting

The DB only runs `docker/db-init/*.sql` on a **fresh** volume. After a schema
change: `docker compose down -v && docker compose up -d --build`.

| Symptom | Cause / fix |
| --- | --- |
| `exec /usr/local/bin/docker-entrypoint.sh: no such file or directory`, api restarts forever | CRLF from a Windows checkout. Fixed by `.gitattributes` plus the `sed` in the Dockerfile. If it persists: `git add --renormalize . && git commit && docker compose build --no-cache api`. |
| Build sits on *transferring context* for GB, or dies with *no space left on device* | Root `.dockerignore` missing or not excluding `**/target/` and `**/node_modules/`. |
| Web build fails on `@rollup/rollup-linux-x64-gnu` or `lightningcss` | Host `node_modules` leaked into the image. `web.Dockerfile` must copy only `src/`, `public/` and the configs, never the whole `aegis-console/`. |
| `SQLX_OFFLINE=true but there is no cached data for this query` | The SQL text of that macro changed after the last `cargo sqlx prepare`. The cache is keyed by SHA-256 of the query string, so even a whitespace edit orphans the old entry. Regenerate as above. |
| `cargo sqlx prepare`: *no driver found for URL scheme* | Reinstall sqlx-cli with `--features rustls,postgres --force`, and use the `postgres://` scheme, not `postgresql://`. |
| DB init: *unrecognized configuration parameter "transaction_timeout"* | Dump is from a newer Postgres than the container - match the `postgres:NN` tag. |
| Runtime/seed: *relation "users" does not exist* | `01_schema.sql` missing or empty -> `docker compose down -v && up -d --build`. |
| `seed` fails: *network not found* | Use `docker compose --profile seed run --rm seed` (one-off), not `up seed`. |
| Port 8080 in use | Change `8080:80` under the `web` service. |
| `--mount=type=cache` parse error | BuildKit disabled. `DOCKER_BUILDKIT=1 docker compose build`, or delete the two `--mount` lines. |

Verify the schema loaded before seeding:

```bash
docker compose exec db psql -U aegis -d aegis -c '\dt'
```

## Before anything beyond local use

Rotate `REFRESH_SECRET` and the Postgres password in `docker-compose.yml`, and
set `DB_SSL_MODE` to `require`/`verify-full` with a real certificate.
