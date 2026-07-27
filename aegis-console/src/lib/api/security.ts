/* Admin SIEM endpoints. Mirrors admin::interface::routes.
 *
 * Two alignment fixes live here:
 *   1. The handlers double-wrap their payload (`data.events`, `data.metrics`,
 *      `data.alerts`); we unwrap the inner key.
 *   2. The backend serializes severity/event_type UPPERCASE ("CRITICAL",
 *      "REFRESH_REPLAY_DETECTED") but the UI keys colors off the lowercase
 *      Severity union. We normalize severity to lowercase on ingest — for BOTH
 *      the REST reads and the live stream — so everything downstream is uniform.
 */

import { api } from "@/lib/api/client";
import type { SecurityAlert, SecurityEvent, SecurityMetrics, Severity } from "@/types";

const SEVS = new Set<Severity>(["info", "low", "medium", "high", "critical"]);

export function normSeverity(raw: unknown): Severity {
    const s = String(raw ?? "").toLowerCase();
    return (SEVS.has(s as Severity) ? s : "info") as Severity;
}

/** Map a raw streamed row (row_to_json from Postgres) into a typed event. */
export function normalizeEvent(raw: Record<string, unknown>): SecurityEvent {
    return {
        id: String(raw.id ?? crypto.randomUUID()),
        user_id: (raw.user_id as number | null) ?? null,
        event_type: String(raw.event_type ?? "unknown"),
        severity: normSeverity(raw.severity),
        ip_address: raw.ip_address == null ? null : String(raw.ip_address),
        user_agent: raw.user_agent == null ? null : String(raw.user_agent),
        session_id: raw.session_id == null ? null : String(raw.session_id),
        jti: raw.jti == null ? null : String(raw.jti),
        family_id: raw.family_id == null ? null : String(raw.family_id),
        metadata: (raw.metadata as Record<string, unknown>) ?? {},
        created_at: String(raw.created_at ?? new Date().toISOString()),
    };
}

export function normalizeAlert(a: SecurityAlert): SecurityAlert {
    return { ...a, severity: normSeverity(a.severity) };
}

interface EventsEnvelope { events: SecurityEvent[] }
interface MetricsEnvelope { metrics: SecurityMetrics }
interface AlertsEnvelope { alerts: SecurityAlert[] }

export const securityApi = {
    events: (limit = 200) =>
        api
            .get<EventsEnvelope>(`/admin/security/events?limit=${limit}`)
            .then((r) => (r?.events ?? []).map((e) => ({ ...e, severity: normSeverity(e.severity) }))),

    metrics: () =>
        api.get<MetricsEnvelope>("/admin/security/metrics").then((r) => r?.metrics ?? null),

    alerts: () =>
        api.get<AlertsEnvelope>("/admin/security/alerts").then((r) => (r?.alerts ?? []).map(normalizeAlert)),

    /** SSE paths (consumed via lib/api/sse.ts). Events are true push (Postgres
     *  LISTEN/NOTIFY); alerts are derived aggregates refreshed on a short cadence. */
    eventsStreamPath: "/admin/security/events/stream",
    alertsStreamPath: "/admin/security/alerts/stream",
    alertsWsPath: "/admin/security/alerts/ws",
};
