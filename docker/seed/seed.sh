#!/bin/sh
# Best-effort demo seed: register admin + victim via the API (correct password
# hashing), then promote the admin. Admin still enrolls MFA on first login.
set -e
apk add --no-cache curl >/dev/null 2>&1 || true
API="http://api:3000"

echo "[seed] waiting for API to accept requests..."
i=0
while [ $i -lt 60 ]; do
    code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$API/register" \
        -H 'Content-Type: application/json' \
        -d '{"email":"__probe__@seed.local","password":"ProbePass123!"}' 2>/dev/null || echo 000)
    [ "$code" != "000" ] && break
    i=$((i+1)); sleep 2
done

reg() {
    curl -s -o /dev/null -w "%{http_code}" -X POST "$API/register" \
        -H 'Content-Type: application/json' \
        -d "{\"email\":\"$1\",\"password\":\"$2\"}" >/dev/null 2>&1 \
        && echo "[seed] registered $1" || echo "[seed] $1 may already exist"
}
reg "admin@test.com"  "AdminPass123!"
reg "victim@test.com" "VictimPass123!"

echo "[seed] promoting admin@test.com to admin role..."
psql -h db -U aegis -d aegis \
    -c "UPDATE users SET user_role='admin' WHERE email='admin@test.com';" \
    || echo "[seed] promote failed (run the UPDATE manually)"

echo "[seed] done."
echo "[seed]   admin : admin@test.com  / AdminPass123!  (enroll MFA on first login)"
echo "[seed]   victim: victim@test.com / VictimPass123!"
