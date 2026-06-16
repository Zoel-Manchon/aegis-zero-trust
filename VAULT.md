# Vault — dynamic database credentials

Instead of the static `aegis/aegis` password baked into `docker-compose.yml`, the
API can obtain **short-lived, uniquely-generated Postgres credentials from
HashiCorp Vault**. Each API container gets its own database role, issued on boot
and set to expire — so a leaked credential is useless after its lease, and every
instance is independently revocable.

## Run it

```bash
docker compose -f docker-compose.yml -f docker-compose.vault.yml up -d --build
docker compose --profile seed run --rm seed       # seed as usual
# → http://localhost:8080      Vault UI → http://localhost:8200  (token: root)
```

Without the overlay file, the stack runs exactly as before (static creds) — Vault
is fully opt-in.

## How it works

```
vault (dev) ──► vault-init configures the database secrets engine
                  │  connection 'aegis'  → connects to Postgres as the admin role
                  │  role       'aegis-api' → mints leased logins with DML grants
                  ▼
api entrypoint ──► GET /v1/database/creds/aegis-api
                  → { username: "v-token-aegis-api-xxxx", password: "...", lease }
                  → DATABASE_URL = postgres://<that user>:<pass>@db:5432/aegis
                  → exec the API
```

1. **`vault`** runs in dev mode (in-memory, auto-unsealed, root token `root`).
2. **`vault-init`** (`vault/init.sh`) enables the database engine, registers the
   Postgres connection (Vault logs in as the `aegis` admin so it can `CREATE
   ROLE`), and defines the `aegis-api` role with creation/revocation SQL and a
   1h default TTL (24h max).
3. The **API entrypoint** (`aegis-api/docker-entrypoint.sh`), seeing `VAULT_ADDR`
   + `VAULT_TOKEN`, requests `database/creds/aegis-api`, builds `DATABASE_URL`
   from the leased username/password, and starts the API with it.

Inspect a freshly-minted credential yourself:

```bash
docker compose exec vault sh -c 'VAULT_ADDR=http://127.0.0.1:8200 VAULT_TOKEN=root vault read database/creds/aegis-api'
# then see the role exists in Postgres:
docker compose exec db psql -U aegis -d aegis -c "\du" | grep v-
```

## Scope & limitations (lab)

- **Dev-mode Vault** is in-memory and auto-unsealed with a known root token —
  fine for a lab, **not** production. A real deployment uses a sealed Vault with
  auth methods (AppRole/Kubernetes), TLS, and audit devices.
- Credentials are fetched **once at container start**. The lease TTL is 1h; for a
  long-running instance you'd renew the lease or re-fetch on expiry. The natural
  next step is a small in-process Vault client in the Rust app (lease renewal +
  pool reconnection) — the entrypoint approach here keeps the app code unchanged
  while still demonstrating per-instance dynamic secrets.
- The root token and admin password are dev defaults; replace them before any
  non-local use.
