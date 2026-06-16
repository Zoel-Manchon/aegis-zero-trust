# Hardening consolidation — exact steps

Closes the three gaps from the audit and expands the attack simulator:

1. Run the **passkeys migration** (current build blocker).
2. Install the **security middleware** (headers/CORS/body-limit/timeout).
3. **Disable passkey login routes** until real WebAuthn verification lands.
4. Drop in the **expanded attack simulator** (20 attacks with PASS/FAIL).
5. Add a **security headers integration test**.

No new folders to create — everything fits in your existing structure.

---

## Files in this archive → target paths

```
core/middleware/security_layer.rs   → src/core/middleware/security_layer.rs        (NEW)
passkeys/interface/routes.rs        → src/modules/passkeys/interface/routes.rs     (REPLACE)
main.rs                             → src/main.rs                                  (REPLACE)
bin/attack_simulator.rs             → src/bin/attack_simulator.rs                  (REPLACE)
tests/common_app.rs                 → tests/common/app.rs                          (REPLACE)
tests/integration/security_headers_test.rs → tests/integration/security_headers_test.rs (NEW)
```

## Step 1 — Run the missing migration (UNBLOCKS THE BUILD)

```bash
cd ~/projects/aegis-api
psql "postgresql://postgres:1234@localhost:5432/testdb" \
  -f src/modules/passkeys/migration/0005_passkey_credentials.sql
psql "postgresql://postgres:1234@localhost:5432/testdb" -c "\d passkey_credentials"
```

This alone fixes the `relation "passkey_credentials" does not exist` build error.

## Step 2 — Cargo.toml: add tower-http features

If `tower-http` isn't in your dependencies, add it with the features the
security layer needs:

```toml
tower-http = { version = "0.6", features = ["cors", "limit", "timeout", "set-header"] }
```

(`tower` itself is pulled in transitively by axum.)

## Step 3 — Place the files

```bash
# Place security middleware
cp core/middleware/security_layer.rs src/core/middleware/security_layer.rs

# Replace passkey routes (disables login endpoints)
cp passkeys/interface/routes.rs src/modules/passkeys/interface/routes.rs

# Replace main.rs (wires the security layer)
cp main.rs src/main.rs

# Replace the attack simulator (expanded battery)
cp bin/attack_simulator.rs src/bin/attack_simulator.rs

# Replace common test app (wires security layer for tests too)
cp tests/common_app.rs tests/common/app.rs

# Add the security headers test
cp tests/integration/security_headers_test.rs tests/integration/security_headers_test.rs
```

## Step 4 — Declare the security_layer module

`src/core/middleware/mod.rs` — add the new line:
```rust
pub mod rate_limit;
pub mod security_layer;   // <-- ADD
```

## Step 5 — Register the new test

`tests/integration.rs` — append:
```rust
#[path = "integration/security_headers_test.rs"]
mod security_headers_test;
```

## Step 6 — Env knobs (optional — defaults are sensible)

```
# .env
APP_ENV=development                      # set to "production" to enable HSTS + strict CORS
CORS_ALLOWED_ORIGINS=https://your-frontend.example  # required when APP_ENV=production
MAX_BODY_BYTES=1048576                   # 1 MiB
REQUEST_TIMEOUT_SECS=15
```

In production you MUST set `CORS_ALLOWED_ORIGINS` or all cross-origin requests
are denied (this is intentional fail-safe behaviour).

## Step 7 — Build and test

```bash
cargo build 2>&1 | tail -10
cargo test 2>&1 | tail -10
```

Expected: all existing tests pass + 1 new test
(`security_headers_test::security_headers_are_present_on_responses`).

## Step 8 — Run the expanded attack simulator

```bash
# in one terminal:
cargo run

# in another:
cargo run --bin attack_simulator
```

You should see 20 PASS lines and a summary like:
```
RESULT: 20 defended, 0 FAILED
All attacks were defended.
```

If any FAIL: that's a real finding. Common ones:
- A19 fails → security middleware not loading (check mod declaration, main.rs)
- A20 fails → passkey routes not replaced (you'd be running with the unsafe login)
- A16/A18 fail → tower-http features missing in Cargo.toml

## Step 9 — Commit (and please initialize git if you haven't)

```bash
cd ~/projects/aegis-api
git init 2>/dev/null
git add -A
git commit -m "harden: install security layer, disable passkey login until WebAuthn, expand simulator to 20 attacks"
```

---

## What this changes about the project posture

- HTTP responses now carry HSTS (prod), X-Frame-Options, X-Content-Type-Options,
  Referrer-Policy, Permissions-Policy, and a restrictive CSP.
- Cross-origin requests are gated by env-configurable CORS.
- Oversized bodies are rejected (DoS resistance).
- Slow/hung requests are timed out globally.
- Passkey login can no longer be exploited as a credential-id-only bypass.
- The simulator covers 20 attack categories with explicit PASS/FAIL.

This brings you to a genuinely defensible baseline. From here, the next big
features (WebAuthn real crypto, Vault, K8s, frontend) all build on a system
that holds up to a basic pentest.
