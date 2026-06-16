# Password Reset feature — install guide

Zero-trust password recovery: no user enumeration, hashed single-use tokens,
short TTL, and full session revocation on reset. The raw token is delivered
out-of-band (logged for now; swap to email later) and NEVER returned by the API.

## Files in this archive

```
migration/0002_password_reset_tokens.sql                  -> run against testdb (and prod db)
auth/infrastructure/repositories/password_reset_repository.rs
                                                          -> src/modules/auth/infrastructure/repositories/
auth/application/password_reset_service.rs                -> src/modules/auth/application/
auth/interface/http/dto/password_reset_request.rs         -> src/modules/auth/interface/http/dto/
auth/interface/http/handlers/password_reset_handler.rs    -> src/modules/auth/interface/http/handlers/
auth/infrastructure/repositories/user_repository_ADD.rs   -> paste its method into existing user_repository.rs
tests/integration/password_reset_test.rs                  -> tests/integration/
```

## 1. Apply the migration

```bash
psql "postgresql://postgres:1234@localhost:5432/testdb" -f migration/0002_password_reset_tokens.sql
# verify
psql "postgresql://postgres:1234@localhost:5432/testdb" -c "\d password_reset_tokens"
```

## 2. Copy the new source files (preserving paths)

```bash
cp auth/infrastructure/repositories/password_reset_repository.rs \
   src/modules/auth/infrastructure/repositories/
cp auth/application/password_reset_service.rs \
   src/modules/auth/application/
cp auth/interface/http/dto/password_reset_request.rs \
   src/modules/auth/interface/http/dto/
cp auth/interface/http/handlers/password_reset_handler.rs \
   src/modules/auth/interface/http/handlers/
cp tests/integration/password_reset_test.rs tests/integration/
```

## 3. Register the new modules (edit 4 existing mod.rs files)

`src/modules/auth/application/mod.rs` — add:
```rust
pub mod password_reset_service;
```

`src/modules/auth/infrastructure/repositories/mod.rs` — add:
```rust
pub mod password_reset_repository;
```

`src/modules/auth/interface/http/dto/mod.rs` — add:
```rust
pub mod password_reset_request;
pub use password_reset_request::{ForgotPasswordRequest, ResetPasswordRequest};
```

`src/modules/auth/interface/http/handlers/mod.rs` — add:
```rust
pub mod password_reset_handler;
```

## 4. Add `update_password` to UserRepository

Open `src/modules/auth/infrastructure/repositories/user_repository.rs` and paste
the method from `user_repository_ADD.rs` inside the existing `impl UserRepository`
block (next to `create_user`).

## 5. Mount the routes

In `src/modules/auth/interface/http/routes.rs`:

- Add the handler import to the `auth_handler` use block, e.g. add a line:
  ```rust
  use crate::modules::auth::interface::http::handlers::password_reset_handler::{
      handler_forgot_password, handler_reset_password,
  };
  ```
- Add the two PUBLIC routes (no auth middleware — the token IS the auth):
  ```rust
  Router::new()
      .route("/register", post(handler_reg_user))
      .route("/login", post(handler_login_user))
      .route("/refresh", post(handler_refresh))
      .route("/password/forgot", post(handler_forgot_password))   // NEW
      .route("/password/reset", post(handler_reset_password))     // NEW
      .merge(protected)
  ```

## 6. Wire the test

`tests/integration.rs` — add:
```rust
#[path = "integration/password_reset_test.rs"]
mod password_reset_test;
```

The test crate also needs `sha2` and `hex` available. They're in `[dependencies]`
already; if `cargo test` complains they're not found in the test crate, add them
under `[dev-dependencies]` with the same versions.

## 7. Build & test

```bash
cargo build 2>&1 | tee /tmp/build.log
cargo test password_reset 2>&1 | tee /tmp/test.log
```

## Security notes (what makes this zero-trust)

- **No enumeration**: `/password/forgot` returns the same body for known and
  unknown emails. `request_reset` returns `Ok(())` either way.
- **Hashed at rest**: only SHA-256(token) is stored. A DB leak yields no usable
  links. (SHA-256 is fine here because the token is already 256-bit random — we
  are not stretching a weak secret.)
- **Single-use**: `mark_used` updates `WHERE used_at IS NULL`, so a token works
  exactly once even under concurrency (last-writer-loses returns false).
- **Short TTL**: 30 minutes, enforced in `perform_reset`.
- **One live token**: requesting a new link invalidates prior unused ones.
- **Session kill on reset**: all sessions revoked after a successful reset, so a
  compromised account cannot persist.
- **Token never in response**: delivered only via `deliver_reset_link` (log now,
  email later). The test seeds its own token row directly to drive the flow.

## TODO when the email/alerts module exists

Replace the body of `deliver_reset_link` in `password_reset_service.rs` with a
real templated email send. Nothing else changes.
