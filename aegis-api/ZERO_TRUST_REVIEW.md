# Zero Trust Auth Review

## Verdict

The project has a strong Zero Trust foundation: RS256 JWTs, refresh rotation, session-backed tokens, token-family revocation, MFA, RBAC middleware, Redis risk counters, audit/security events, admin metrics, and an SSE alert stream.

It is not fully hardened yet. The next work should focus on phishing-resistant MFA, stricter policy enforcement, secret protection, real-time event architecture, Redis hardening, and SIEM-grade data modeling.

## Highest-priority implementation backlog

1. **Passkeys / WebAuthn hardware-key support**
   - Add WebAuthn as the preferred MFA/step-up method.
   - Keep TOTP as fallback, not the strongest factor.
   - Support security keys and platform authenticators.
   - Store credential public keys, sign counters, transports, backup eligibility, and user verification policy.

2. **MFA hardening**
   - Encrypt TOTP secrets using Vault/KMS envelope encryption.
   - Add last-used TOTP timestep to prevent replay inside the same 30-second window.
   - Add per-user/per-MFA-token attempt lockout.
   - Add hashed recovery codes.
   - Require step-up before disabling MFA.

3. **RBAC -> policy engine hardening**
   - Current path-prefix RBAC is OK for MVP but weak for scale.
   - Make authorization method-aware: `GET /admin/security/events` is not equivalent to `DELETE /admin/users/:id`.
   - Create typed permissions and route registration tests that fail if an admin route lacks an explicit permission.
   - Add ABAC: tenant, resource owner, device trust, risk score, MFA age, session age.
   - Use deny-by-default for unknown protected paths.

4. **Password reset hardening**
   - Do not log raw reset tokens outside local development.
   - Actually check reset lockout before calling the reset service.
   - Add reset-token audit events: requested, consumed, expired/rejected, session-revoked.

5. **Redis hardening**
   - Use TLS and ACL credentials.
   - Prefix all keys by environment/app.
   - Treat Redis outage as a security event.
   - Decide fail-open/fail-closed behavior by control type.
   - Use Lua scripts for atomic multi-key lockout decisions where needed.

6. **Real-time alert stream hardening**
   - SSE endpoint is currently protected by `auth_middleware`, but should explicitly require `AdminAccess`/`AlertRead`.
   - Add max stream duration and disconnect telemetry.
   - Move from DB polling to event push: Redis Stream, Postgres LISTEN/NOTIFY, NATS, Kafka, or broadcast channel.
   - Never stream raw secrets, tokens, request bodies, or excessive metadata.

7. **Dashboard/SIEM data model**
   - Add query filters: time range, actor, IP, user agent, event type, severity, session_id, jti, family_id, rule_id.
   - Add timeline view per user/session/token family.
   - Add correlation rules: impossible travel, refresh replay + MFA failure, password reset + new device, admin access from new ASN.
   - Add alert lifecycle: open, acknowledged, escalated, resolved, false positive.

8. **Attack simulator**
   - Expanded in `src/bin/attack_simulator.rs`.
   - It now probes auth bypass, RBAC bypass, SSE access, refresh replay, malformed refresh, MFA burst, reset spray, device churn, and Redis counters.

## Kubernetes before or after frontend?

Build the **frontend/SIEM MVP before Kubernetes**, unless you specifically need Kubernetes for deployment learning or production parity right now.

Recommended order:

1. Harden auth primitives and policy engine.
2. Build the SIEM frontend against stable dashboard/event APIs.
3. Add Vault integration for JWT keys, refresh secret, DB credentials, TOTP encryption keys.
4. Containerize cleanly with Docker Compose for local integration.
5. Move to Kubernetes.
6. Add Vault dynamic DB credentials, external secrets, network policies, ingress mTLS/OIDC, pod security, and observability.

Reason: Kubernetes amplifies operational complexity. If the product surface and event model are still changing, Kubernetes will slow iteration. Once frontend/API contracts stabilize, Kubernetes becomes valuable.

## Feature recommendation: password with hardware key?

Yes, but call it **passkeys/WebAuthn**, not “password with hardware key.” The ideal model is:

- Password + WebAuthn for high-risk users/admins.
- Passkey-first/passwordless as a future mode.
- TOTP as fallback only.
- Step-up WebAuthn for dangerous actions: disable MFA, revoke all sessions, view secrets, admin operations, export logs.


## Added feature: Passkeys / hardware-key login

New module: `src/modules/passkeys`.

Endpoints added:

- `POST /passkeys/register/begin` — protected, starts WebAuthn registration.
- `POST /passkeys/register/finish` — protected, stores credential metadata.
- `GET /passkeys` — protected, lists current user's passkeys.
- `DELETE /passkeys` — protected, revokes one passkey.
- `POST /passkeys/login/begin` — public, starts passwordless login.
- `POST /passkeys/login/finish` — public, finishes login and issues tokens.

Important: the WebAuthn module is intentionally shaped as a first-class auth factor, but the cryptographic attestation/assertion verification is marked as a hardening TODO. Before production, wire this to `webauthn-rs` or equivalent and enforce:

- challenge equality and one-time challenge use;
- exact origin and RP ID checks;
- user presence and user verification;
- algorithm allowlist;
- signature verification over authenticator data and client data hash;
- sign-counter clone detection;
- generic errors for enumeration resistance;
- event emission into audit/SIEM for registration, login, failure, revocation, and cloned-authenticator suspicion.

## Recommended next security features

1. Real WebAuthn verification with passkeys/security keys.
2. Step-up policy engine: require passkey/MFA for admin, RBAC changes, passkey deletion, password reset, and suspicious risk scores.
3. Device registry: trusted devices, device revocation, last seen, ASN/country drift.
4. Strong RBAC hardening: role hierarchy, deny-by-default permissions, scoped resources, admin action approvals.
5. Session posture scoring: continuously recompute session trust from IP churn, UA drift, velocity, failed MFA, policy denials.
6. Redis hardening: TLS, ACLs, key prefix isolation, fail-closed behavior for auth-critical counters.
7. SIEM normalization: event schema, correlation IDs, MITRE ATT&CK tags, severity score, entity timeline.
8. Alert rules: refresh replay, MFA bombing, passkey clone counter regression, impossible travel, policy-denied burst, admin SSE access attempts.
9. Recovery hardening: recovery codes, passkey recovery flow, mandatory recent-auth check before adding/removing factors.
10. Secrets phase: HashiCorp Vault for JWT signing keys, refresh pepper, DB credentials, MFA encryption keys, SMTP/API secrets.

## Frontend vs Kubernetes order

Build the frontend/SIEM MVP before Kubernetes. You need the security workflows visible first: event explorer, alert triage, session graph, identity timeline, RBAC admin panel, and passkey enrollment UX. Then add Vault. Kubernetes should come after the app's security model and operational surfaces are coherent, unless you need K8s now for deployment parity.
