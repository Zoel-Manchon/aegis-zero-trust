# Auth ports & adapters (additive, zero-risk)

This adds the `UserRepository` and `SessionRepository` **ports** (traits) and
their Postgres **adapters** to the auth module. It is purely additive:

- No existing file's behaviour changes.
- Existing services (`auth_service`, `refresh_service`) keep calling the
  concrete repos exactly as before.
- The full test suite stays green.

The ports are adopted **incrementally**: the new password-recovery service
(next step) will be built directly on these traits, proving them on new code.
Migrating `auth_service`/`refresh_service` onto them happens later, in an
isolated pass, so we never bet the working baseline on a big-bang rewrite.

## Files (6 new)

```
auth/application/ports/mod.rs                          NEW
auth/application/ports/user_repository.rs              NEW  (trait)
auth/application/ports/session_repository.rs           NEW  (trait)
auth/infrastructure/adapters/mod.rs                    NEW
auth/infrastructure/adapters/pg_user_repository.rs     NEW  (adapter)
auth/infrastructure/adapters/pg_session_repository.rs  NEW  (adapter)
```

## Apply

From the project root, extract the tarball over `src/modules/auth/`:

```bash
cd ~/projects/aegis-api
mkdir -p /tmp/authstage && tar -xzf ~/Downloads/auth_ports.tar.gz -C /tmp/authstage
cp -r /tmp/authstage/auth/. src/modules/auth/
```

Then declare the two new submodules in the existing mod files:

```bash
# application/mod.rs -> add `pub mod ports;`
grep -q 'pub mod ports;' src/modules/auth/application/mod.rs || \
  echo 'pub mod ports;' >> src/modules/auth/application/mod.rs

# infrastructure/mod.rs -> add `pub mod adapters;`
grep -q 'pub mod adapters;' src/modules/auth/infrastructure/mod.rs || \
  echo 'pub mod adapters;' >> src/modules/auth/infrastructure/mod.rs
```

## Wire into AppState (optional now, required for recovery)

Add the two repositories to `AppState` so handlers/services can share one
instance. In `src/app_state.rs`:

```rust
use crate::modules::auth::application::ports::{
    session_repository::SessionRepository, user_repository::UserRepository,
};
use crate::modules::auth::infrastructure::adapters::{
    pg_session_repository::PgSessionRepository, pg_user_repository::PgUserRepository,
};

// in the struct:
    pub users: Arc<dyn UserRepository>,
    pub sessions: Arc<dyn SessionRepository>,

// in AppState::new(...), after building risk adapters:
    let users: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(pool.clone()));
    let sessions: Arc<dyn SessionRepository> = Arc::new(PgSessionRepository::new(pool.clone()));
// ...and include `users, sessions` in the returned struct literal.
```

(If you prefer, skip the AppState wiring until we build password recovery —
the ports/adapters compile fine on their own.)

## Verify

```bash
cargo build
cargo test
```

Expected: clean build, all tests still pass. If `async_trait` isn't already a
dependency it will error — but it is (the risk ports use it), so this should
just work.
