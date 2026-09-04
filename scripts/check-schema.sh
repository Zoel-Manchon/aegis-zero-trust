#!/usr/bin/env bash
# =============================================================================
# Schema drift check.
#
# There is no auto-migrator: `docker/db-init/01_schema.sql` is what a fresh
# database gets, and `aegis-api/src/**/migration/*.sql` are applied by hand to
# existing ones. Those two have to describe the same schema, and nothing
# enforces it — so the day a migration lands without regenerating the dump,
# every `docker compose down -v` produces a database the API cannot run
# against, with an error as unhelpful as:
#
#     column "last_used_step" does not exist
#
# This script catches that. It builds a scratch database from the committed
# init files, applies every migration on top, and fails if anything changed:
# if a migration still has work to do against a fresh schema, the dump is
# stale.
#
# Usage (the stack must be up):
#     bash scripts/check-schema.sh
#
# Regenerate the dump when it fails:
#     docker compose exec -T db pg_dump -U aegis -d aegis \
#         --schema-only --no-owner --no-privileges > docker/db-init/01_schema.sql
# =============================================================================
set -euo pipefail

DB_USER="${DB_USER:-aegis}"
SCRATCH="${SCRATCH:-aegis_schema_check}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

psql_scratch() { docker compose exec -T db psql -U "$DB_USER" -d "$SCRATCH" -v ON_ERROR_STOP=1 -q "$@"; }
psql_admin()   { docker compose exec -T db psql -U "$DB_USER" -d postgres -v ON_ERROR_STOP=1 -q "$@"; }

cleanup() { psql_admin -c "DROP DATABASE IF EXISTS $SCRATCH" >/dev/null 2>&1 || true; }
trap cleanup EXIT

cd "$ROOT"

echo "==> building a scratch database from the committed init files"
cleanup
psql_admin -c "CREATE DATABASE $SCRATCH" >/dev/null
for file in docker/db-init/*.sql; do
    echo "    $file"
    psql_scratch < "$file" >/dev/null
done

echo "==> dumping it"
docker compose exec -T db pg_dump -U "$DB_USER" -d "$SCRATCH" \
    --schema-only --no-owner --no-privileges > /tmp/schema-before.sql

echo "==> applying every migration on top"
# Sorted by file name, which is what the numeric prefixes are for.
while IFS= read -r migration; do
    echo "    $migration"
    psql_scratch < "$migration" >/dev/null
done < <(find aegis-api/src -name '*.sql' -path '*migration*' | sort)

echo "==> dumping it again"
docker compose exec -T db pg_dump -U "$DB_USER" -d "$SCRATCH" \
    --schema-only --no-owner --no-privileges > /tmp/schema-after.sql

# The dump carries a random \restrict token and a timestamp header; neither is
# schema, and both would make every run look like a drift.
normalise() { grep -v -E '^\\(restrict|unrestrict) |^-- (Dumped|Started)' "$1"; }

if diff -u <(normalise /tmp/schema-before.sql) <(normalise /tmp/schema-after.sql) > /tmp/schema-drift.diff; then
    echo
    echo "OK — the committed schema already contains every migration."
else
    echo
    echo "DRIFT — a migration changes a freshly initialised database, which means"
    echo "docker/db-init/01_schema.sql is behind. A clone or a 'down -v' would"
    echo "produce a database the API cannot run against."
    echo
    sed -n '1,60p' /tmp/schema-drift.diff
    exit 1
fi
