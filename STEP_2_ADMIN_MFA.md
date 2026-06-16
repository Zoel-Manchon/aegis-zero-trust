# Step 2 — Mandatory TOTP MFA for admins

Admins must have an **enabled** TOTP enrollment to use the Security Operations
Console (and the attack range). Without it, every admin/attack-range endpoint
returns `403 ADMIN_MFA_REQUIRED` and the dashboard shows an enrollment gate.

## How it's enforced

```
request → auth_middleware (verifies JWT+session, inserts SecurityContext)
        → admin_mfa_guard (route layer, just inside auth)
              if role == Admin and MFA not enabled  ⇒  403 ADMIN_MFA_REQUIRED
        → handler
```

- New `admin_mfa_guard` middleware (`auth/interface/middleware/admin_mfa_guard.rs`)
  layered onto **admin_routes** and **attack_range_routes**, inside the auth
  layer so it can read the role and check enrollment via the same
  `mfa_repository::find_by_user_id(...).enabled` the `/me` endpoint uses.
- New `AppError::AdminMfaRequired` → `403`, code `ADMIN_MFA_REQUIRED`. It's a 403
  (not 401), so the client does **not** trigger a token refresh — no loop.
- Non-admins are untouched (their role is already enforced by the policy engine
  for `/admin` and by `require_admin` for the attack range).

## Frontend

- `useSecurityData` catches `ADMIN_MFA_REQUIRED`, stops the streams/polls, and
  exposes `mfaRequired`.
- The dashboard renders an **enrollment gate** (“set up MFA →”, links to
  `/account`) instead of the SOC.

## Flow for an admin

1. Admin without MFA signs in → reaches `/dashboard` → sees the MFA gate.
2. Clicks **set up MFA** → `/account` → scans the QR, enters a code (existing
   MFA setup UI).
3. **Sign in again** so the session is MFA-authenticated; from now on every admin
   login is MFA-challenged (existing behavior for enrolled users), and the SOC
   opens normally.

## Verification

- **Frontend: verified** (tsc clean, vite build green).
- **Backend: NOT compiled here** (no Rust toolchain). Run `cargo build && cargo test`.
  New/changed: `admin_mfa_guard.rs` (new), `middleware/mod.rs` (+module),
  `core/errors/app_error.rs` (+`AdminMfaRequired` variant & mapping),
  `admin/interface/routes.rs` + `attack_range/interface/routes.rs` (guard layer).

## Notes / honest gaps

- Enforcement is on *enrollment* (MFA enabled), plus login-time MFA challenge for
  enrolled users. It does not yet track a per-session "completed MFA this session"
  flag — the one soft spot is the very first session right after enrolling, which
  is why the gate says to sign in again.
- Still not done from your list: **geolocation on real logins** ("location from
  the user log") — currently geo is driven by the attack range; wiring it into the
  login handler is the next small backend add.
