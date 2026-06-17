# Passkeys (WebAuthn)

Aegis supports passwordless, phishing-resistant login with **passkeys**. A passkey
is a public/private keypair generated and held by the user's authenticator (Touch
ID / Windows Hello / a hardware key). The private key never leaves the device; the
server stores only the **public** credential. All cryptographic verification is
performed server-side by [`webauthn-rs`](https://crates.io/crates/webauthn-rs) — no
challenge or signature is ever trusted blindly.

## Flow

Both ceremonies are two round-trips. The in-progress state
(`PasskeyRegistration` / `PasskeyAuthentication`) is persisted in **Redis** between
begin and finish — `webauthn-rs` requires this to block replay.

```
Registration (enrol a device, while signed in)
  POST /passkeys/register/begin   -> { challenge_id, public_key }     # CreationChallengeResponse
  navigator.credentials.create(public_key)                            # device signs, makes keypair
  POST /passkeys/register/finish  { challenge_id, credential }        # verified -> Passkey stored

Authentication (sign in)
  POST /passkeys/login/begin      { email } -> { challenge_id, public_key }   # RequestChallengeResponse
  navigator.credentials.get(public_key)                              # device signs the challenge
  POST /passkeys/login/finish     { challenge_id, credential }        # signature verified -> JWT pair
```

The browser-side base64url ⇄ ArrayBuffer marshalling uses
[`@github/webauthn-json`](https://github.com/github/webauthn-json); the helpers live
in `aegis-console/src/lib/auth/webauthn.ts`.

## What the server verifies

`finish_passkey_registration` / `finish_passkey_authentication` assert the
challenge, the **origin** (phishing resistance), the RP ID hash, user
presence/verification, the attestation (registration) or assertion **signature**
(login), and — on login — that the credential's **signature counter** advanced
(clone detection). Only the verified public `Passkey` blob and its counter are
persisted; the stored counter is advanced after each successful assertion.

## Configuration

The relying-party identity comes from env (defaults shown — they match the local
Caddy origin and work out of the box):

| Var | Default | Notes |
|-----|---------|-------|
| `WEBAUTHN_RP_ID` | `localhost` | Must be a registrable suffix of the origin host. |
| `WEBAUTHN_RP_ORIGIN` | `http://localhost:8080` | **Must equal the browser's origin.** |
| `WEBAUTHN_RP_NAME` | `Aegis` | Shown by some authenticators. |

For a real domain over HTTPS, set e.g. `WEBAUTHN_RP_ID=auth.example.com` and
`WEBAUTHN_RP_ORIGIN=https://auth.example.com`. A mismatch between the origin and the
browser is the most common cause of a failed ceremony.

## Storage

`passkey_credentials` holds the credential id, the serialized public `Passkey`
(public key + counter), friendly name, transports, and timestamps. It never holds a
private key or any reusable secret.
