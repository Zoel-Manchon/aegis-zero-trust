# aegis — change log (this pass)

Two projects: `aegis-api/` (Rust/axum backend) and
`aegis-console/` (React + Vite + Tailwind v4 SOC frontend).

The frontend was fully type-checked and production-built in CI for this pass
(`tsc -b` clean, `vite build` succeeds). The backend was edited surgically but
**not compiled here** (no Rust toolchain in the build sandbox) — run
`cargo build && cargo test` locally to confirm.

---

## Backend changes (`aegis-api/`)

1. **`/refresh` now returns the rotated `jti`.**
   - `auth/application/refresh_service.rs`: `refresh_token()` now returns
     `(access, refresh, jti)` instead of `(access, refresh)`.
   - `auth/interface/http/handlers/refresh_handler.rs`: destructures the new
     jti and adds it to the JSON response.
   - Single caller (the handler); the replay test only reads login's jti, so
     nothing else is affected.

2. **New `GET /me` (protected).** Returns `{ user_id, email, role,
   mfa_enabled, risk_score }`. The access-token JWT carries no role, so a SPA
   otherwise can't learn its identity/role without an admin-only call.
   - New `auth/interface/http/handlers/me_handler.rs`.
   - Registered in `handlers/mod.rs` and wired into the protected group in
     `auth/interface/http/routes.rs`.

3. **Removed the orphan `src/passkeys/` directory** — a dead duplicate of
   `src/modules/passkeys/...` not referenced by `lib.rs`.

4. **`.gitignore`** extended to cover DB dumps (`*.backup`, `test_dump.sql`).

### Backend follow-ups you should do (not code changes)
- **Rotate secrets and the RSA keypair.** `.env` (REFRESH_SECRET,
  ACCESS_TOKEN_SECRET), `private_key.pem`, and `public_key.pem` are tracked in
  git history, so they're compromised. After rotating, untrack them:
  `git rm --cached .env private_key.pem public_key.pem` (they're already in
  `.gitignore` going forward).
- Consider returning a consistent envelope from `/refresh` (it's the only
  endpoint that returns bare tokens).

---

## Frontend changes (`aegis-console/`)

Full modular refactor aligned to the backend modules. React + Vite + Tailwind
v4 + Recharts kept; added `qrcode` for MFA enrollment.

### Bugs fixed
- **Silent-refresh self-destruct (critical).** The backend rotates the session
  jti on every refresh; the old SPA kept the original jti and tripped
  replay-detection on the 2nd refresh — force-logout + a false `CRITICAL`
  `refresh_replay_detected` in the SIEM. Now the rotated jti is taken from the
  refresh response, or decoded from the new access token if the backend predates
  fix #1 (`lib/auth/jwt.ts`, `lib/auth/AuthContext.tsx`).
- **Admin endpoint shape mismatch.** `/admin/security/{events,metrics,alerts}`
  double-wrap their payload (`data.events`, …). `lib/api/security.ts` now
  unwraps the inner key.
- **Register min-length** corrected from 8 → 12 to match the backend policy.
- **401 handling.** The api client now runs a single-flight silent refresh and
  retries once on 401 (`lib/api/client.ts`).

### New / rebuilt
- **Dashboard** is now real: metrics + events polled from the admin API, alerts
  live over the authenticated SSE stream (consumed via `fetch`, since
  `EventSource` can't send a bearer token). Split into a data hook
  (`useSecurityData`), pure derivations (`derive.ts`), and panels.
- **MFA challenge** page (post-login step-up), wired to `/mfa/complete-login`.
- **Account → Security** self-service: MFA enrollment with a locally-generated
  QR + disable, passkey list/revoke (enrollment flagged experimental since the
  backend WebAuthn verifier is stubbed), and "sign out everywhere".
- **Email verification** and **password reset/forgot** pages.
- **Role gating**: the dashboard is wrapped in `RoleRoute require="admin"`;
  identity/role come from `/me`. Degrades gracefully if `/me` isn't deployed.

### Structure
```
src/
  app/        router, ProtectedRoute, RoleRoute, AppLayout
  components/ Panel, Stat, SevDot, Field, Button, Notice, AuthShell, severity
  lib/api/    client, sse, auth, mfa, passkeys, security
  lib/auth/   AuthContext, jwt
  types/      common, auth, mfa, passkeys, security (+ index barrel)
  features/
    auth/     Login, Register, MfaChallenge, ForgotPassword, ResetPassword, VerifyEmail
    account/  AccountPage, MfaSection, PasskeysSection, SessionsSection
    security/ Dashboard, useSecurityData, derive, panels/*
```

---

## Run order
1. **Backend**: set up Postgres + Redis, then `cargo build && cargo test`,
   then run. Listens on `127.0.0.1:3000` by default.
2. **Frontend**: `npm install` then `npm run dev` (Vite on `:5173`, proxies
   `/api/*` → `:3000`). For prod, set `VITE_API_BASE` to the real API origin.

---

## Phase 1 add-on — real-time SOC + red/blue loop

See **DEMO_RUNBOOK.md** for the full workflow. Summary of changes:

**Backend (verify with `cargo build && cargo test`):**
- `src/migration/0007_soc_event_notify.sql` — trigger that `pg_notify`s the full
  row on every `security_events` insert. **Apply this migration** or the live
  feed stays empty. (No changes to the audit-chain insert code.)
- New `admin/.../handlers/security_events_stream_handler.rs` — SSE that holds a
  `PgListener` on `soc_events` and pushes events live. Wired at
  `GET /admin/security/events/stream` (admin-only).
- `attack_simulator` gained a continuous **storm** subcommand:
  `cargo run --bin attack_simulator storm [--rps N --secs N --victims N]`. The
  default (no subcommand) still runs the one-shot pentest battery.

**Frontend (tsc + build verified):**
- **Casing alignment fix**: backend emits UPPERCASE severity/event_type; the UI
  keyed colors off lowercase. `lib/api/security.ts` now normalizes severity on
  ingest for both REST reads and the live stream.
- `useSecurityData` now consumes the live event stream (true push) instead of
  polling events; alerts stream on a short cadence; metrics poll (aggregate).

**Still queued (Phase 2):** blue-team response actions — revoke session family,
lock IP, acknowledge alert.

---

## Alerts: real-time fix + popups + sound + geo map (frontend, verified)

- **Real-time bug fixed.** The 5s aggregate alert poll was *replacing* the whole
  alert list, wiping just-pushed WebSocket alerts. Live (WS) and derived
  (aggregate) alerts are now separate arrays merged for display, so live alerts
  are genuinely real-time and never clobbered. Events were already true-push (SSE
  via Postgres NOTIFY); alerts are now true-push via WS.
- **Popups now actually pop.** `AlertToast` is portaled to `<body>` (nothing
  clips it), stacks newest-on-top, animates in, and auto-dismisses.
- **Sound.** Each live alert plays a synthesized chime (Web Audio, no asset),
  urgent triple-tone for criticals. A 🔊/🔇 toggle sits in the dashboard header.
  (Browsers may require one page interaction before audio is allowed.)
- **Geolocation on the dashboard.** New `GeoMapPanel` plots GeoIP origins on an
  equirectangular map with red arcs for impossible-travel hops.
- **Attack console.** The Attack Range panel now keeps a rolling **launch log**
  (recent throws + IMPOSSIBLE_TRAVEL hits) — an operator console, not a one-shot.

NOTE: these need the Step-1 backend compiled/running to show live data — the
launch endpoint + WS alert dispatch live there. Still pending: **Step 2 —
mandatory TOTP MFA for admins** (backend enforcement; not yet implemented).

---

## Step 2 — mandatory TOTP MFA for admins

See STEP_2_ADMIN_MFA.md. New `admin_mfa_guard` route layer on admin + attack-range
routes denies admins without enabled MFA via `403 ADMIN_MFA_REQUIRED`; the
dashboard shows an enrollment gate linking to `/account`. Backend NOT compiled
here (run `cargo build && cargo test`); frontend verified.

---

## Real-login geolocation + Step 3 (SOC reporter)

**Real-login geolocation (backend — run `cargo build`):**
- `geo/login.rs` (new) `record_login_geo(state, user_id, ip, ua)` geo-locates a
  successful login, runs impossible-travel detection (records IMPOSSIBLE_TRAVEL +
  dispatches a WS alert if tripped), and returns the `geoip` metadata.
- `security_audit::login_success` now takes `geoip: Option<Value>` and embeds it
  in the LoginSuccess event (so real logins plot on the map). Wired into **both**
  login paths: password (`auth_handler`) and MFA-completion (`mfa_handler`).
- Loopback/private IPs (0,0) are skipped — to see geo locally, send a public IP
  via `X-Forwarded-For` (the server already reads it).

**Step 3 — SOC reporter (frontend, verified):**
- `LiveFeedPanel` upgraded into a SOC reporter view: live counts header
  (total/crit/high), inline **geolocation** column (city, country), critical-row
  emphasis, monospace, and a streaming/paused indicator.

Pending: Step 4 design pass (cohesive map/popups), Step 5 Docker, Step 6 Vault.

---

## Step 4 — design pass (cohesion)

Frontend, verified. Kept the Bloomberg/terminal aesthetic; improved structure:
- The dashboard is now organized into labeled zones — **RED TEAM** (launch
  console), **TELEMETRY** (rate/severity), **GEO INTELLIGENCE** (map +
  impossible-travel + geoip grouped together), **OPERATIONS** (feed + alerts) —
  via a new `SectionLabel`.
- Replaced the dashboard's ad-hoc header with a slim **console sub-bar** (status
  pill + sound + pause). The global brand/nav/clock already live in `AppLayout`,
  so the sub-bar no longer duplicates them.
- Footer now honestly reads `GeoIP: ACTIVE` (real-login geo is wired).

Remaining: Step 5 Docker (Caddy + seed admin/victim), Step 6 Vault dynamic creds.

---

## Step 5 — Docker

Full stack via `docker compose` (see **DOCKER.md**): Postgres + Redis + the Rust
API + Caddy serving the console on a **single origin** (Caddy proxies `/api/*` →
API, so SSE/WS/cookies and `X-Forwarded-For` geo all work). The API image is a
multi-stage Rust build; an entrypoint generates the JWT keypair on first boot
into a volume. A `--profile seed` one-shot registers admin/victim and promotes
the admin.

Two one-time prereqs (documented): drop a `pg_dump --schema-only` at
`docker/db-init/01_schema.sql` (no auto-migrator), and run `cargo sqlx prepare`
to produce the `.sqlx` offline cache (compile-time macros). Validated here:
compose YAML + shell syntax. NOT run (no Docker/Rust in this environment).

Remaining: Step 6 — HashiCorp Vault dynamic DB credentials.
