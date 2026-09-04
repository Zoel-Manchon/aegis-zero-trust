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

`vault-init` also enables the **transit** engine and creates the `aegis-mfa`
key, which wraps the per-seed data keys that encrypt TOTP secrets at rest. With
the overlay the API runs with `MFA_KEY_WRAPPER=vault`, so the key material never
reaches the API process or the database — Vault only ever sees a 32-byte data
key. Watch a wrap happen while enrolling a device:

```bash
docker compose exec vault sh -c 'VAULT_ADDR=http://127.0.0.1:8200 VAULT_TOKEN=root vault read transit/keys/aegis-mfa'
docker compose exec db psql -U aegis -d aegis -c 'SELECT user_id, left(secret, 24) FROM user_mfa'
```

The second command is the point: what is in the column is `aegis.v1....`, not a
base32 seed. Without the overlay the base stack falls back to `MFA_KEK`, a known
development key set in `docker-compose.yml` — same envelope, weaker custody.

Dev-mode Vault is in-memory with a known root token: a lab demonstration, not a
production posture. Credentials are fetched once at container start (1h lease);
a real deployment would renew the lease in-process.

## Trusting Caddy's certificate

Caddy serves `https://localhost` with a certificate from **its own internal
CA**, which lives in the `aegis_caddy_data` volume. `caddy-root.crt` in the
repository root is a copy of that CA as it stood when it was committed.

`docker compose exec web caddy trust` installs it into the **container's** trust
store, which is not where your browser looks. To trust it on the host, export
the root and import that file:

```bash
# Export whatever CA the running Caddy actually has right now
docker compose exec -T web cat /data/caddy/pki/authorities/local/root.crt > caddy-root.crt

# Confirm which one you are looking at
openssl x509 -in caddy-root.crt -noout -subject -dates -fingerprint -sha256
```

Then import `caddy-root.crt`:

- **Firefox** - Settings > Privacy & Security > Certificates > *View
  Certificates* > **Authorities** > Import > tick *Trust this CA to identify
  websites*. Firefox keeps its own store; adding the CA to Windows does nothing
  for it.
- **Chrome / Edge / Windows** - `certutil -addstore -user Root caddy-root.crt`,
  or Manage Computer Certificates > Trusted Root Certification Authorities.

### When the CA is regenerated

Deleting `aegis_caddy_data` - which is what `docker compose down -v` does -
makes Caddy mint a **new** CA on the next boot. The new one carries the *same
subject name* as the old, because the name only encodes the year:

```text
CN=Caddy Local Authority - 2026 ECC Root
```

So a browser that still trusts the old CA finds it by name, tries to verify the
new certificate's signature with the old public key, and fails. Firefox reports
this as **`SEC_ERROR_BAD_SIGNATURE`** - *bad signature*, not *unknown issuer*,
which is the tell that a stale CA with a colliding name is in the way rather
than none at all.

Recovering is two steps, and the order matters:

1. **Delete the old authority first.** Firefox > *View Certificates* >
   **Authorities** > *Caddy Local Authority* > **Delete or Distrust**. Importing
   the new root without removing the old one leaves two CAs with the same
   subject, and the failure can persist.
2. Export and import the new root with the commands above.

HSTS makes this non-optional. The Caddyfile sends
`Strict-Transport-Security: max-age=31536000`, so once Firefox has seen it the
*Advanced > Accept the Risk* escape hatch is no longer offered for `localhost` -
the certificate has to actually validate. To clear that state, use **Forget
About This Site** on `localhost` from the History sidebar (it also clears
cookies and history for the origin).

The cheapest fix, of course, is not regenerating the CA at all: keep
`aegis_caddy_data` and drop only the volume you actually meant to reset.

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
change, drop *that* volume and nothing else:

```bash
docker compose down
docker volume rm aegis_db-data
docker compose up -d --build
```

**Do not reach for `docker compose down -v`.** It takes every named volume with
it, including `aegis_caddy_data` — which holds Caddy's internal CA — and
`aegis_api-keys`, which holds the JWT keypair. Losing the first breaks HTTPS in
a browser that trusted the old CA (see the certificate row below); losing the
second invalidates every access token ever issued.

| Symptom | Cause / fix |
| --- | --- |
| `exec /usr/local/bin/docker-entrypoint.sh: no such file or directory`, api restarts forever | CRLF from a Windows checkout. Fixed by `.gitattributes` plus the `sed` in the Dockerfile. If it persists: `git add --renormalize . && git commit && docker compose build --no-cache api`. |
| Build sits on *transferring context* for GB, or dies with *no space left on device* | Root `.dockerignore` missing or not excluding `**/target/` and `**/node_modules/`. |
| Web build fails on `@rollup/rollup-linux-x64-gnu` or `lightningcss` | Host `node_modules` leaked into the image. `web.Dockerfile` must copy only `src/`, `public/` and the configs, never the whole `aegis-console/`. |
| `SQLX_OFFLINE=true but there is no cached data for this query` | The SQL text of that macro changed after the last `cargo sqlx prepare`. The cache is keyed by SHA-256 of the query string, so even a whitespace edit orphans the old entry. Regenerate as above. |
| `cargo sqlx prepare`: *no driver found for URL scheme* | Reinstall sqlx-cli with `--features rustls,postgres --force`, and use the `postgres://` scheme, not `postgresql://`. |
| DB init: *unrecognized configuration parameter "transaction_timeout"* | Dump is from a newer Postgres than the container - match the `postgres:NN` tag. |
| Runtime/seed: *relation "users" does not exist* | `01_schema.sql` missing or empty -> drop only the DB volume: `docker compose down && docker volume rm aegis_db-data && docker compose up -d --build`. |
| Firefox: **`SEC_ERROR_BAD_SIGNATURE`** at `https://localhost` (Chrome: `ERR_CERT_AUTHORITY_INVALID`) | `docker compose down -v` deleted `aegis_caddy_data`, so Caddy minted a **new** internal CA. Its subject name is identical to the old one, so the browser picks the CA it already trusts and fails to verify a signature made by a different key — hence *bad signature* rather than *unknown issuer*. Delete the old **Caddy Local Authority** from the browser and import the new root: see [Trusting Caddy's certificate](#trusting-caddys-certificate). |
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

Replace `MFA_KEK` too, or better, run with the Vault overlay so the KEK lives in
transit instead of an environment variable. Generate one with
`openssl rand -base64 32`. If neither is configured the API still boots, logs a
warning, and stores TOTP seeds in plaintext — the pre-existing behaviour, made
explicit rather than silent.
