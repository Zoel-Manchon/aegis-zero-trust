# aegis — Demo Runbook (red team vs. blue team)

This is the spine of the project. Read it once and the whole thing makes sense.

## What this is, in one sentence

A **zero-trust identity provider with a built-in SOC**: the backend authenticates
users and records every security-relevant event into a tamper-evident audit log;
the admin SOC console visualizes that telemetry live and correlates it into
alerts.

## The cast

| Role | Who | Sees |
| --- | --- | --- |
| **End user** | a normal registered account (role `user`) | only their own account (MFA, passkeys, sessions). **Not** the SOC. |
| **Blue team** | an admin account (role `admin`) | the SOC dashboard — live event feed, metrics, alerts. |
| **Red team** | `attack_simulator` | nothing — it's a tool that *attacks the API*. |

End users are the **attack surface**, not operators. That's why only admins see
the dashboard — exactly like a real IdP (Okta's admin console vs. the end-user app).

## The loop

```
RED TEAM                       SYSTEM UNDER TEST                BLUE TEAM (admin SOC)
attack_simulator storm ──────▶ auth API (zero-trust)  ────────▶ security_events  (audit, hash-chained)
 failed logins,                defends every request,            │  Postgres NOTIFY on insert
 refresh replays,              records what happened             │        │
 RBAC bypass,                                                    │        ▼
 alg:none forgery,                                               │   SSE push ──▶ SOC dashboard
 enumeration probes                                              │                (live feed · metrics · alerts)
```

Events are **true push**: a Postgres trigger `NOTIFY`s on every insert, and the
admin SSE endpoint relays it instantly. No polling on the hot path.

---

## Run it (three terminals)

### 0. One-time setup
- Postgres + Redis running; `.env` pointed at them (**rotate the committed
  secrets first — see CHANGES.md**).
- Apply migrations, including the new SOC trigger:
  `src/migration/0007_soc_event_notify.sql` (creates the `NOTIFY` trigger that
  makes the live feed work). Apply it the same way you apply your other
  migrations (psql or your migration runner).

### 1. Backend (the system under test)
```bash
cd aegis-api
cargo build && cargo test      # verify this pass's edits compile + pass
cargo run                      # listens on 127.0.0.1:3000
```

### 2. Frontend (the blue-team console)
```bash
cd aegis-console
npm install
npm run dev                    # http://localhost:5173 (proxies /api -> :3000)
```

### 3. Make yourself blue team
Register an account in the UI, then promote it in Postgres:
```sql
UPDATE users SET user_role = 'admin' WHERE email = 'you@example.com';
```
Sign in again → you land on **/dashboard** (regular users land on /account).

### 4. Red team — run the storm
```bash
cd aegis-api
cargo run --bin attack_simulator storm                 # defaults: 4 req/s, 60s, 5 victims
cargo run --bin attack_simulator storm --rps 8 --secs 180 --victims 10
```
Watch the dashboard: the event feed scrolls in real time, the event-rate chart
climbs, the severity mix shifts, brute-force lockouts and refresh-replay
**criticals** fire, and the alerts panel lights up.

For the one-shot pentest report instead of continuous traffic:
```bash
cargo run --bin attack_simulator                       # 20 attacks, PASS/FAIL summary
```

---

## Talk track (90 seconds)

1. "This is a zero-trust auth service. Every request re-verifies the token,
   re-checks the session in the DB, evaluates live risk, and enforces policy —
   nothing is trusted twice."
2. "Everything it sees lands in a hash-chained audit log. Here's the SOC console
   reading it live." *(show the quiet dashboard)*
3. "Now the red team." *(run `storm`)* "These are real attacks against the API —
   credential stuffing, refresh-token replay, privilege-escalation attempts,
   `alg:none` token forgery."
4. "The system defends every one, and the blue team sees it happen in real
   time." *(point at the rising rate, the criticals, the alerts)*
5. "Replay a stolen refresh token and the whole session family is revoked and
   flagged critical — that's the token-theft tripwire."

---

## What's real vs. still scaffold

- **Real:** auth + sessions + refresh rotation/replay defense, hash-chained
  audit, risk engine, rate limiting, the pentest battery, live event push, the
  SOC dashboard.
- **Stubbed:** WebAuthn/passkey *verification* (enrollment UI is flagged
  experimental).
- **Phase 2 (next):** blue-team **response actions** — revoke a session family,
  lock an IP, acknowledge an alert — so the console becomes operational, not
  just observational.

## Architecture notes

- Live events: `security_events` INSERT → trigger `pg_notify('soc_events', row)`
  → `GET /admin/security/events/stream` (SSE via `PgListener`) → dashboard.
- Alerts: derived aggregates (`derived_security_alerts`) streamed every few
  seconds — correct for 10-minute / 24-hour windows.
- Auth for SSE uses a `fetch`-based reader (not `EventSource`) so the bearer
  token can be sent.
