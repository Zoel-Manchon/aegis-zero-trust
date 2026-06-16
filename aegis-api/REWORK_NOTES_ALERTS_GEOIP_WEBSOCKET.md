# Rework notes — alerts, WebSocket admin console, GeoIP, impossible travel

This ZIP contains a targeted rework of the broken SOC/admin alignment.

## Implemented

- Added **real-time WebSocket alert delivery** for the admin dashboard:
  - Backend route: `GET /admin/security/alerts/ws`
  - Backend channel: `BroadcastAlertChannel` using `tokio::sync::broadcast`
  - Frontend client: `src/lib/api/ws.ts`
  - Dashboard now consumes WebSocket push alerts and keeps the old SSE aggregate stream as fallback.

- Fixed the alert path:
  - `AlertDispatcher` now fans out to log, email stub, Redis latest-alert key, and WebSocket broadcast.
  - Admin alert frames are normalized to the dashboard `SecurityAlert` shape.

- Added **GeoIP enrichment scaffolding**:
  - Module: `src/modules/geo/mod.rs`
  - Events now receive `metadata.geoip` with country, city, coordinates, ASN, and network type.
  - This is offline/deterministic fallback logic; replace it with MaxMind DB in production without changing the dashboard metadata contract.

- Added **impossible travel detection**:
  - On each audited security event with user + IP, the service compares the current GeoIP with the user's previous login/MFA/refresh GeoIP.
  - If speed exceeds `900 km/h`, event metadata gets `metadata.impossible_travel.detected = true` and severity is upgraded to `CRITICAL`.
  - The admin derived-alert query now emits `IMPOSSIBLE_TRAVEL` alerts.

- Added dashboard panels:
  - `GeoIpPanel`
  - `ImpossibleTravelPanel`
  - WebSocket status remains visible in the SOC header.

- Added an **admin terminal attack/simulation console**:
  - `cargo run --bin admin_terminal -- --base-url http://127.0.0.1:3000`
  - Options include login failure, impossible-travel simulation, policy denial, and invoking the pentest battery.

- Added support for proxy/test source IPs:
  - `X-Forwarded-For`, `X-Real-IP`, and `CF-Connecting-IP` are accepted by the request IP extractor.
  - This makes local simulation possible without external clients.

## Important runtime notes

- The WebSocket route uses the existing auth middleware. Browser WebSockets cannot set `Authorization`, so the frontend appends `?access_token=<token>` and the middleware accepts that token for stream routes.
- For production, put the API behind a trusted proxy and only honor forwarded IP headers from that proxy.
- The existing Postgres LISTEN/NOTIFY event SSE still requires running `src/migration/0007_soc_event_notify.sql` if you want event-row push. Alert push no longer depends on that migration.

## Validation I could not run here

This environment does not have `cargo` or installed frontend `node_modules`, so I could not execute `cargo check` or `npm run typecheck`. The changes were made directly against the source tree and documented here.
