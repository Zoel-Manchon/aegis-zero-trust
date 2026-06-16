# Step 1 — GeoIP impossible-travel + launch-from-origin + WS popups

This is the auth-lab red→blue loop, ported to the Rust stack: pick an attacker
**origin**, **launch** a scenario, watch events stream into the SOC, and trip
**IMPOSSIBLE_TRAVEL** by launching from two distant origins in a row — with a
live **WebSocket popup** alert.

## How it works (the pipeline)

```
Dashboard "Attack Range"  ──POST /attack-range/launch {scenario, origin, victim_email}
        │
        ▼
attack_range_service::launch
   • resolve origin (preset key or IP) → geo::lookup → GeoIpInfo
   • write scenario events (attributed to victim + origin IP + metadata.geoip)
        └─▶ security_events INSERT ─▶ trigger pg_notify('soc_events', row)
                                          └─▶ SSE /admin/security/events/stream ─▶ live feed + Geo/Travel panels
   • dispatch scenario Alert ─▶ AlertDispatcher ─▶ BroadcastAlertChannel ─▶ alert_bus
                                                       └─▶ WS /admin/security/alerts/ws ─▶ toast popup
   • geo::travel::evaluate(redis, victim, geo)  (Redis geo:last:{user_id})
        rule: distance > 100 km AND speed > 900 km/h  ⇒ IMPOSSIBLE_TRAVEL
        └─▶ IMPOSSIBLE_TRAVEL event (Critical) + Critical Alert (red popup)
```

GeoIP is an **offline resolver** (no MaxMind/.mmdb/license). The origin presets
map representative IPs to real city coordinates, which is all the demo needs and
runs anywhere (incl. Docker later).

## API contract (admin-only)

`GET /attack-range/scenarios`
```json
{ "data": {
  "scenarios": [{ "key": "brute_force", "label": "Brute force", "description": "…" }, …],
  "origins":   [{ "key": "madrid", "label": "Madrid, ES", "ip": "31.10.10.10" }, …]
} }
```

`POST /attack-range/launch`  body `{ "scenario", "origin", "victim_email" }` →
```json
{ "data": {
  "scenario": "brute_force", "origin_ip": "133.10.10.10",
  "origin": { "country": "JP", "city": "Tokyo", "lat": 35.68, "lon": 139.65 },
  "events_recorded": 6, "impossible_travel": true,
  "distance_km": 10760, "speed_kmh": 64560,
  "from": { "country": "ES", "city": "Madrid", "lat": 40.42, "lon": -3.70 }
} }
```

**Event metadata shape** (matches the frontend `derive.ts` exactly):
- every attack-range event: `metadata.geoip = { ip, country, city, latitude, longitude, network_type, asn }`
- the impossible-travel event also: `metadata.impossible_travel = { detected: true, distance_km, speed_kmh, from:{country,city}, to:{country,city} }`

## Demo (≈2 minutes)

1. Backend: `cd aegis-api && cargo build && cargo test && cargo run`.
   Apply migrations incl. `src/migration/0007_soc_event_notify.sql` (the events
   feed needs that trigger).
2. Frontend: `cd aegis-console && npm install && npm run dev`.
3. Register a **victim** (e.g. `victim@test.com`) and your own account in the UI.
4. Promote yourself: `UPDATE users SET user_role = 'admin' WHERE email = 'you@example.com';`
   then sign in again → `/dashboard`.
5. In **Attack Range**: origin **Madrid**, scenario **Brute force**, victim
   `victim@test.com` → **launch**. Events stream into the feed; a HIGH alert
   toast pops; GeoIP panel shows Madrid.
6. Immediately launch again with origin **Tokyo**, same victim →
   **IMPOSSIBLE_TRAVEL**: a red critical toast, an entry in the Impossible-Travel
   panel (Madrid → Tokyo, ~64,000 km/h), and the critical event in the feed.

## What changed

**Backend (NOT compiled here — run `cargo build && cargo test`):**
- `audit/domain/security_event.rs` — added `ImpossibleTravel` → `"IMPOSSIBLE_TRAVEL"`.
- `geo/origins.rs` (new) — attacker-origin presets. `geo/travel.rs` (new) —
  Redis-backed impossible-travel evaluator (the `km>100 && speed>900` rule).
  `geo/mod.rs` — declares the two submodules.
- `modules/attack_range/**` (new) — service + admin-only handlers + routes
  (`/attack-range/scenarios`, `/attack-range/launch`).
- `modules/mod.rs`, `main.rs` — register + merge the module.

**Frontend (verified — tsc + vite build clean):**
- `lib/api/attackRange.ts` (new), `features/security/AttackRange.tsx` (new) —
  the launch control.
- `components/AlertToast.tsx` (new) + `useSecurityData` now exposes `latestAlert`
  — WS alerts pop as toasts.
- Backend metadata aligned to the existing `GeoIpPanel` / `ImpossibleTravelPanel`
  (they were already built but had no real data to show).

## Honest caveats / next

- Geo currently drives off the **attack-range** path. Wiring it into **real
  logins** ("location from the user log") is a small follow-on in the login
  handler — slated next to Step 2.
- Still queued: **Step 2** mandatory TOTP MFA for admins · **Step 3** richer
  attack console · **Step 4** design/map polish · **Step 5** Docker · **Step 6** Vault.
- Cleanup noted: the dead `RedisStreamChannel` remains in the dispatcher
  (harmless); the SSE alerts stream is now redundant with the WS path (kept as a
  fallback for now).
