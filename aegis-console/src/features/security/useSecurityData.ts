/* =============================================================================
 * useSecurityData — the dashboard's data engine.
 *
 *   - events: initial fetch + TRUE PUSH stream (Postgres LISTEN/NOTIFY → SSE).
 *   - alerts: split into two sources that no longer fight each other —
 *       liveAlerts   = real-time WebSocket pushes (what pops + chimes)
 *       derivedAlerts= periodic aggregate recompute (10m/24h windows)
 *     The old bug: the aggregate poll REPLACED the whole list every 5s, wiping
 *     just-pushed live alerts. Now they're separate arrays, merged for display,
 *     so live alerts are genuinely real-time and persist.
 *   - metrics: all-time counts, polled.
 * ========================================================================== */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { securityApi, normalizeEvent, normalizeAlert } from "@/lib/api/security";
import { ApiClientError } from "@/lib/api/client";
import { streamSSE } from "@/lib/api/sse";
import { streamWS } from "@/lib/api/ws";
import { useAuth } from "@/lib/auth/AuthContext";
import type { SecurityAlert, SecurityEvent, SecurityMetrics } from "@/types";

const METRICS_POLL_MS = 8000;
const EVENT_LIMIT = 200;
const MAX_EVENTS = 300;

export type StreamStatus = "open" | "reconnecting" | "closed";

interface AlertWsFrame {
    type: "hello" | "alert" | "lagged";
    alert?: SecurityAlert;
}

export function useSecurityData() {
    const { refresh } = useAuth();
    const [metrics, setMetrics] = useState<SecurityMetrics | null>(null);
    const [events, setEvents] = useState<SecurityEvent[]>([]);
    const [liveAlerts, setLiveAlerts] = useState<SecurityAlert[]>([]);
    const [derivedAlerts, setDerivedAlerts] = useState<SecurityAlert[]>([]);
    const [latestAlert, setLatestAlert] = useState<{ alert: SecurityAlert; nonce: number } | null>(null);
    const [status, setStatus] = useState<StreamStatus>("closed");
    const [paused, setPaused] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [mfaRequired, setMfaRequired] = useState(false);

    const refreshRef = useRef(refresh);
    refreshRef.current = refresh;

    // Display list: live pushes first (each kept), then aggregates not already
    // represented live. Live alerts are never clobbered by the aggregate poll.
    const alerts = useMemo(() => {
        const liveTypes = new Set(liveAlerts.map((a) => a.alert_type));
        return [...liveAlerts, ...derivedAlerts.filter((d) => !liveTypes.has(d.alert_type))].slice(0, 24);
    }, [liveAlerts, derivedAlerts]);

    const pull = useCallback(async () => {
        try {
            const [m, e, a] = await Promise.all([
                securityApi.metrics(),
                securityApi.events(EVENT_LIMIT),
                securityApi.alerts(),
            ]);
            setMetrics(m);
            setEvents(e);
            setDerivedAlerts(a);
            setMfaRequired(false);
            setError(null);
        } catch (e) {
            if (e instanceof ApiClientError && e.code === "ADMIN_MFA_REQUIRED") {
                setMfaRequired(true);
                return;
            }
            setError("Couldn't load SIEM data. Confirm the admin role and that the backend is up.");
        }
    }, []);

    useEffect(() => {
        void pull();
    }, [pull]);

    // aggregate refresh (metrics + derived alerts) — own arrays, no clobber
    useEffect(() => {
        if (paused || mfaRequired) return;
        const t = window.setInterval(() => {
            void securityApi.metrics().then(setMetrics).catch(() => {});
            void securityApi.alerts().then(setDerivedAlerts).catch(() => {});
        }, METRICS_POLL_MS);
        return () => window.clearInterval(t);
    }, [paused, mfaRequired]);

    // live event push (true real-time)
    useEffect(() => {
        if (paused || mfaRequired) {
            setStatus("closed");
            return;
        }
        const handle = streamSSE<Record<string, unknown>>(securityApi.eventsStreamPath, {
            eventName: "soc_event",
            onMessage: (raw) => setEvents((prev) => [normalizeEvent(raw), ...prev].slice(0, MAX_EVENTS)),
            onStatus: setStatus,
            onUnauthorized: () => refreshRef.current(),
        });
        return () => handle.close();
    }, [paused, mfaRequired]);

    // live alert push (true real-time) — WebSocket only; this is what pops/chimes
    useEffect(() => {
        if (paused || mfaRequired) return;
        const ws = streamWS<AlertWsFrame>(securityApi.alertsWsPath, {
            onMessage: (frame) => {
                if (frame.type !== "alert" || !frame.alert) return;
                const alert = normalizeAlert(frame.alert);
                setLatestAlert({ alert, nonce: Date.now() + Math.random() });
                setLiveAlerts((prev) => [alert, ...prev].slice(0, 16));
            },
            onUnauthorized: () => refreshRef.current(),
        });
        return () => ws.close();
    }, [paused, mfaRequired]);

    return { metrics, events, alerts, latestAlert, status, paused, setPaused, error, mfaRequired, reload: pull };
}
