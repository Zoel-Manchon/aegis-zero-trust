/* Pure, client-side derivations over the real event stream. The backend
 * supplies aggregate metrics but not these breakdowns, so we compute them from
 * the fetched events. Kept pure so they're trivially testable. */

import type { Severity, SecurityEvent } from "@/types";
import { SEVERITIES } from "@/components/severity";

export interface TimeBucket {
    t: number;
    count: number;
    crit: number;
}

/** Events-per-minute for the last 30 minutes, plus a high/critical overlay. */
export function timeline(events: SecurityEvent[]): TimeBucket[] {
    const buckets: TimeBucket[] = Array.from({ length: 30 }, (_, i) => ({ t: i, count: 0, crit: 0 }));
    const now = Date.now();
    for (const e of events) {
        const age = Math.floor((now - new Date(e.created_at).getTime()) / 60000);
        if (age >= 0 && age < 30) {
            const b = buckets[29 - age];
            b.count++;
            if (e.severity === "critical" || e.severity === "high") b.crit++;
        }
    }
    return buckets;
}

export interface SevSlice {
    sev: Severity;
    count: number;
}

export function severityDistribution(events: SecurityEvent[]): SevSlice[] {
    const d: Record<Severity, number> = { info: 0, low: 0, medium: 0, high: 0, critical: 0 };
    for (const e of events) d[e.severity]++;
    return SEVERITIES.map((sev) => ({ sev, count: d[sev] }));
}

export interface IpCount {
    ip: string;
    count: number;
}

export function topIps(events: SecurityEvent[], n = 6): IpCount[] {
    const c = new Map<string, number>();
    for (const e of events) {
        if (e.ip_address) c.set(e.ip_address, (c.get(e.ip_address) ?? 0) + 1);
    }
    return [...c.entries()]
        .sort((a, b) => b[1] - a[1])
        .slice(0, n)
        .map(([ip, count]) => ({ ip, count }));
}


export interface GeoPoint {
    ip: string;
    country: string;
    city: string;
    latitude: number;
    longitude: number;
    count: number;
    network_type: string;
}

export function geoPoints(events: SecurityEvent[], n = 8): GeoPoint[] {
    const c = new Map<string, GeoPoint>();
    for (const e of events) {
        const g = e.metadata?.geoip as Record<string, unknown> | undefined;
        if (!g) continue;
        const key = `${String(g.city ?? "Unknown")}|${String(g.country ?? "UNKNOWN")}`;
        const prev = c.get(key);
        if (prev) { prev.count++; continue; }
        c.set(key, {
            ip: e.ip_address ?? String(g.ip ?? "unknown"),
            country: String(g.country ?? "UNKNOWN"),
            city: String(g.city ?? "Unknown"),
            latitude: Number(g.latitude ?? 0),
            longitude: Number(g.longitude ?? 0),
            network_type: String(g.network_type ?? "unknown"),
            count: 1,
        });
    }
    return [...c.values()].sort((a, b) => b.count - a.count).slice(0, n);
}

export interface ImpossibleTravelHit {
    id: string;
    user_id: number | null;
    ip: string | null;
    speed_kmh: number;
    distance_km: number;
    from: string;
    to: string;
    created_at: string;
}

export function impossibleTravelHits(events: SecurityEvent[]): ImpossibleTravelHit[] {
    return events.flatMap((e) => {
        const t = e.metadata?.impossible_travel as Record<string, unknown> | undefined;
        if (!t || t.detected !== true) return [];
        const from = t.from as Record<string, unknown> | undefined;
        const to = t.to as Record<string, unknown> | undefined;
        return [{
            id: e.id,
            user_id: e.user_id,
            ip: e.ip_address,
            speed_kmh: Number(t.speed_kmh ?? 0),
            distance_km: Number(t.distance_km ?? 0),
            from: `${String(from?.city ?? "Unknown")}, ${String(from?.country ?? "UNKNOWN")}`,
            to: `${String(to?.city ?? "Unknown")}, ${String(to?.country ?? "UNKNOWN")}`,
            created_at: e.created_at,
        }];
    }).slice(0, 5);
}


export interface EventTypeCount {
    type: string;
    label: string;
    count: number;
    hostile: boolean;
}

/* Event types that indicate an attack/anomaly → drawn in the warning color. */
const HOSTILE_TYPES = new Set([
    "LOGIN_FAILURE", "BRUTE_FORCE_LOCKOUT", "CREDENTIAL_STUFFING",
    "REFRESH_REPLAY_DETECTED", "TOKEN_PURPOSE_VIOLATION", "POLICY_DENIED",
    "IMPOSSIBLE_TRAVEL", "DEVICE_FINGERPRINT_MISMATCH", "SESSION_HIJACK",
    "PRIVILEGE_ESCALATION", "MFA_FAILURE", "SESSION_REVOKED",
]);

/** Top event types by volume — the attack-vector breakdown. */
export function eventTypeDistribution(events: SecurityEvent[], n = 8): EventTypeCount[] {
    const c = new Map<string, number>();
    for (const e of events) {
        const t = (e.event_type ?? "UNKNOWN").toUpperCase();
        c.set(t, (c.get(t) ?? 0) + 1);
    }
    return [...c.entries()]
        .sort((a, b) => b[1] - a[1])
        .slice(0, n)
        .map(([type, count]) => ({
            type,
            count,
            hostile: HOSTILE_TYPES.has(type),
            label: type.toLowerCase().replace(/_/g, " "),
        }));
}
