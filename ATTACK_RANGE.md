# Attack Range

The SOC's red-team console. Pick an attacker **origin** (geo preset or raw IP) and
a **scenario**, name a victim account, and **launch** — events stream into the live
feed, alerts pop over the WebSocket bus, and a second launch from a distant origin
trips impossible-travel. Admin-only (behind auth + mandatory-MFA guard).

## Scenarios

Each scenario writes a sequence of attributed `security_events` (hash-chained) and
dispatches a SOC alert. All are relevant to a zero-trust auth surface:

| Key | What it simulates | Notable events |
| --- | --- | --- |
| `brute_force` | Repeated failed logins → lockout | `LOGIN_FAILURE` ×4, `BRUTE_FORCE_LOCKOUT` |
| `credential_stuffing` | High-volume reused-credential attempts | `LOGIN_FAILURE` ×4, `CREDENTIAL_STUFFING`, `BRUTE_FORCE_LOCKOUT` |
| `token_replay` | Reuse of a rotated refresh token | `LOGIN_SUCCESS`, `REFRESH_REPLAY_DETECTED` (critical) |
| `jwt_tamper` | Forged / wrong-purpose access token | `TOKEN_PURPOSE_VIOLATION` |
| `fingerprint_spoof` | Forged device fingerprint to defeat binding | `DEVICE_FINGERPRINT_MISMATCH`, `POLICY_DENIED`, `SESSION_REVOKED` |
| `session_hijack` | Stolen session replayed from a new device | `SESSION_HIJACK` (critical), `TOKEN_PURPOSE_VIOLATION` |
| `mfa_bypass` | Step-up / MFA failures probing for a gap | `MFA_FAILURE` ×2, `POLICY_DENIED` |
| `rbac_bypass` | Normal user reaching an admin route | `POLICY_DENIED` |
| `privilege_escalation` | User attempts to elevate to admin | `PRIVILEGE_ESCALATION` (critical), `SESSION_REVOKED` |
| `storm` | Multi-vector burst — one launch, a wave of events | a dozen events across all of the above |

## Run all

The **▶▶ run all scenarios** button fires every scenario in sequence from the
selected origin (client-side orchestration over the single-launch endpoint), so a
single click populates the whole telemetry picture.

## CLI simulator

For load/realism against the live API there's also a binary that hits real
endpoints:

```bash
cargo run --bin attack_simulator storm --rps 20 --secs 10 --victims 5
```

## Telemetry

The dashboard's **attack vectors** chart breaks events down by type (attack-
indicating types in the warn color), alongside the event-rate and severity charts —
so the scenarios above show up distinctly as you run them.
