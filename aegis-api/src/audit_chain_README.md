# Tamper-evident audit log (hash chaining) — install guide

Makes `security_events` tamper-evident: each event is hash-chained to the
previous one, so any modification, deletion, or reordering breaks the chain and
is detectable. Turns the SIEM dashboard's "chain integrity: VERIFIED ✓" from
aspirational into real.

## Files

```
migration/0004_audit_hash_chain.sql                 -> run against testdb (and prod)
audit/domain_hash_chain.rs                           -> src/modules/audit/domain/hash_chain.rs
audit/chained_audit_repository.rs                    -> src/modules/audit/infrastructure/repositories/chained_audit_repository.rs
tests/integration/audit_chain_test.rs                -> tests/integration/audit_chain_test.rs
```

## 1. Migration

```bash
psql "postgresql://postgres:1234@localhost:5432/testdb" -f src/migration/0004_audit_hash_chain.sql
psql "postgresql://postgres:1234@localhost:5432/testdb" -c "\d security_events"   # confirm seq, prev_hash, event_hash
```

## 2. Copy files (note the domain file is renamed to hash_chain.rs)

```bash
cp audit/domain_hash_chain.rs           src/modules/audit/domain/hash_chain.rs
cp audit/chained_audit_repository.rs    src/modules/audit/infrastructure/repositories/chained_audit_repository.rs
cp tests/integration/audit_chain_test.rs tests/integration/audit_chain_test.rs
```

## 3. Declare the new modules

`src/modules/audit/domain/mod.rs` — add:
```rust
pub mod hash_chain;
```

`src/modules/audit/infrastructure/repositories/mod.rs` — add:
```rust
pub mod chained_audit_repository;
```

## 4. Route writes through the chained insert

In `src/modules/audit/application/audit_service.rs`, change the one call in
`record_event` from the plain insert to the chained one:

```rust
// OLD:
// use crate::modules::audit::infrastructure::repositories::audit_repository;
// ...
// audit_repository::insert_security_event(pool, event).await

// NEW:
use crate::modules::audit::infrastructure::repositories::chained_audit_repository;
// ...
pub async fn record_event(pool: &sqlx::PgPool, event: NewSecurityEvent) {
    if let Err(err) = chained_audit_repository::insert_chained_event(pool, event).await {
        tracing::error!(error = ?err, "failed to write security event");
    }
}
```

(You can keep the old `audit_repository` around or delete it; nothing else uses
it once this is switched.)

## 5. Wire the test

`tests/integration.rs` — add:
```rust
#[path = "integration/audit_chain_test.rs"]
mod audit_chain_test;
```

## 6. (Optional) expose a verify endpoint for the dashboard

Add an admin route that calls `verify_chain(&state.pool)` and returns the
`ChainVerification` JSON, e.g. `GET /admin/security/chain/verify`. The dashboard
footer ("chain integrity") can poll it. Wire it in
`src/modules/admin/interface/routes.rs` alongside the other admin routes with a
small handler:

```rust
pub async fn verify_chain_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = chained_audit_repository::verify_chain(&state.pool)
        .await
        .map_err(|_| AppError::DatabaseError)?;
    Ok(Json(ApiResponse::success(serde_json::to_value(r).unwrap())))
}
```

## 7. Build & test

```bash
cargo build 2>&1 | tee /tmp/build.log
cargo test audit_chain 2>&1 | tee /tmp/test.log
cargo test            # confirm all still green
```

The two new tests prove the headline property:
- `audit_chain_verifies_after_activity` — normal activity yields a clean chain.
- `audit_chain_detects_tampering` — directly editing a row in the DB (simulating
  an attacker) is caught, and verification points at the exact broken seq.

## Design notes (for your write-up / interview)

- **Why it's tamper-EVIDENT, not tamper-PROOF:** an attacker with DB access can
  still alter rows, but cannot do so *silently* — the chain no longer verifies.
  To forge undetectably they'd have to recompute every subsequent hash, which a
  periodic off-box checkpoint (anchoring the latest hash elsewhere) would also
  defeat. That's the standard model and worth stating explicitly.
- **Concurrency:** appends are serialized with a Postgres advisory xact lock so
  the chain stays strictly linear. Throughput cost is one short lock per event;
  fine for an audit log.
- **Canonicalization is a versioned contract:** the field order in
  `hash_chain::canonical_payload` must never change for existing data, or old
  chains won't verify. Documented in the code.
- **created_at is computed app-side** so the hashed timestamp exactly matches
  the stored one (using DB `now()` would differ between hash-compute and store).

## Existing data note
If `security_events` already has rows (from before this change), they'll have
NULL seq/prev_hash/event_hash and are simply skipped by `verify_chain` (which
filters `seq IS NOT NULL`). The chain starts fresh from the first chained insert.
For a clean demo, you can TRUNCATE security_events first.
