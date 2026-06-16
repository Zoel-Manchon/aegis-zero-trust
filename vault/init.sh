#!/bin/sh
# Configures Vault's database secrets engine to mint short-lived Postgres roles
# for the API. Runs once (as the `vault-init` service) against the dev Vault.
set -e

export VAULT_ADDR="${VAULT_ADDR:-http://vault:8200}"
export VAULT_TOKEN="${VAULT_TOKEN:-root}"

echo "[vault-init] waiting for Vault at $VAULT_ADDR ..."
until vault status >/dev/null 2>&1; do sleep 1; done

echo "[vault-init] enabling database secrets engine"
vault secrets enable database 2>/dev/null || echo "[vault-init] database engine already enabled"

# Vault connects to Postgres as the admin role so it can CREATE/DROP roles.
echo "[vault-init] configuring postgres connection 'aegis'"
vault write database/config/aegis \
    plugin_name=postgresql-database-plugin \
    allowed_roles="aegis-api" \
    connection_url="postgresql://{{username}}:{{password}}@${DB_HOST:-db}:${DB_PORT:-5432}/${DB_NAME:-aegis}?sslmode=disable" \
    username="${DB_ADMIN_USER:-aegis}" \
    password="${DB_ADMIN_PASSWORD:-aegis}"

# A dynamic role: each request mints a unique login with DML on the app schema,
# valid for the lease TTL, then revoked + dropped automatically.
echo "[vault-init] creating dynamic role 'aegis-api' (default TTL 1h, max 24h)"
vault write database/roles/aegis-api \
    db_name=aegis \
    creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT USAGE ON SCHEMA public TO \"{{name}}\"; GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO \"{{name}}\"; GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO \"{{name}}\";" \
    revocation_statements="REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM \"{{name}}\"; REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM \"{{name}}\"; REVOKE USAGE ON SCHEMA public FROM \"{{name}}\"; DROP ROLE IF EXISTS \"{{name}}\";" \
    default_ttl="1h" \
    max_ttl="24h"

echo "[vault-init] done. Issue creds with:  vault read database/creds/aegis-api"
