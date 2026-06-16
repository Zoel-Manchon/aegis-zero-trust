#!/bin/sh
# Generates the RSA JWT keypair on first boot (persisted in a volume so tokens
# survive restarts), then launches the API.
set -e

KEY_DIR="${KEY_DIR:-/app/keys}"
PRIV="${JWT_PRIVATE_KEY_PATH:-$KEY_DIR/private_key.pem}"
PUB="${JWT_PUBLIC_KEY_PATH:-$KEY_DIR/public_key.pem}"
mkdir -p "$KEY_DIR"

if [ ! -f "$PRIV" ] || [ ! -f "$PUB" ]; then
    echo "[entrypoint] generating RSA JWT keypair at $KEY_DIR"
    openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$PRIV"
    openssl rsa -pubout -in "$PRIV" -out "$PUB"
fi

export JWT_PRIVATE_KEY_PATH="$PRIV"
export JWT_PUBLIC_KEY_PATH="$PUB"

# --- Optional: dynamic DB credentials from HashiCorp Vault -------------------
# When VAULT_ADDR + VAULT_TOKEN are set, request short-lived Postgres creds from
# Vault's database secrets engine and build DATABASE_URL from them. Without
# them, the statically-provided DATABASE_URL is used unchanged.
if [ -n "$VAULT_ADDR" ] && [ -n "$VAULT_TOKEN" ]; then
    role="${VAULT_DB_ROLE:-aegis-api}"
    db_host="${DB_HOST:-db}"
    db_port="${DB_PORT:-5432}"
    db_name="${DB_NAME:-aegis}"
    echo "[entrypoint] requesting dynamic DB creds from Vault ($VAULT_ADDR, role=$role)"
    user=""
    pass=""
    i=0
    while [ "$i" -lt 30 ]; do
        resp=$(curl -s -H "X-Vault-Token: $VAULT_TOKEN" "$VAULT_ADDR/v1/database/creds/$role" || true)
        user=$(printf '%s' "$resp" | jq -r '.data.username // empty' 2>/dev/null || true)
        pass=$(printf '%s' "$resp" | jq -r '.data.password // empty' 2>/dev/null || true)
        if [ -n "$user" ] && [ -n "$pass" ]; then
            break
        fi
        echo "[entrypoint] waiting for Vault DB role to be ready... ($i)"
        sleep 2
        i=$((i + 1))
    done
    if [ -n "$user" ] && [ -n "$pass" ]; then
        export DATABASE_URL="postgres://${user}:${pass}@${db_host}:${db_port}/${db_name}"
        echo "[entrypoint] using Vault-issued DB user: $user (short-lived, leased)"
    else
        echo "[entrypoint] WARNING: no Vault creds obtained — falling back to static DATABASE_URL"
    fi
fi

echo "[entrypoint] starting aegis on ${SERVER_ADDR:-0.0.0.0:3000}"
exec aegis
