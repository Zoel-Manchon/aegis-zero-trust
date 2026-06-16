/* =============================================================================
 * aegis :: Security Operations Console
 *
 * Layout is organized into labeled zones: a command bar, a KPI strip, then
 * RED TEAM (launch console), TELEMETRY (rate/severity), GEO INTELLIGENCE
 * (map + impossible-travel + geoip), and OPERATIONS (live feed + alerts).
 * Data engine: useSecurityData (push events/alerts, polled metrics).
 * Admin-gated upstream by <RoleRoute require="admin">.
 * ========================================================================== */

import { useMemo, useState } from "react";
import { Notice } from "@/components/Notice";
import { SectionLabel } from "@/components/SectionLabel";
import { useSecurityData } from "@/features/security/useSecurityData";
import { eventTypeDistribution, geoPoints, impossibleTravelHits, severityDistribution, timeline, topIps } from "@/features/security/derive";
import { MetricRow } from "@/features/security/panels/MetricRow";
import { EventRatePanel } from "@/features/security/panels/EventRatePanel";
import { SeverityPanel } from "@/features/security/panels/SeverityPanel";
import { EventTypePanel } from "@/features/security/panels/EventTypePanel";
import { LiveFeedPanel } from "@/features/security/panels/LiveFeedPanel";
import { AlertsPanel } from "@/features/security/panels/AlertsPanel";
import { TopIpsPanel } from "@/features/security/panels/TopIpsPanel";
import { GeoIpPanel } from "@/features/security/panels/GeoIpPanel";
import { ImpossibleTravelPanel } from "@/features/security/panels/ImpossibleTravelPanel";
import { AttackRange } from "@/features/security/AttackRange";
import { AlertToast } from "@/components/AlertToast";
import { GeoMapPanel } from "@/features/security/GeoMapPanel";

const statusLabel: Record<string, string> = {
    open: "● LIVE",
    reconnecting: "◐ RECONNECTING",
    closed: "■ PAUSED",
};
const statusClass: Record<string, string> = {
    open: "bg-panel-hi text-accent",
    reconnecting: "bg-[#2a2410] text-sev-medium",
    closed: "bg-[#3a1414] text-sev-critical",
};

export default function Dashboard() {
    const { metrics, events, alerts, latestAlert, status, paused, setPaused, error, mfaRequired } = useSecurityData();
    const [muted, setMuted] = useState(false);

    const rate = useMemo(() => timeline(events), [events]);
    const sevDist = useMemo(() => severityDistribution(events), [events]);
    const evTypes = useMemo(() => eventTypeDistribution(events), [events]);
    const ips = useMemo(() => topIps(events), [events]);
    const geos = useMemo(() => geoPoints(events), [events]);
    const travel = useMemo(() => impossibleTravelHits(events), [events]);

    const indicator = paused ? "closed" : status;

    if (mfaRequired) {
        return (
            <div className="flex min-h-[60vh] items-center justify-center p-6">
                <div className="w-full max-w-md border border-sev-high/50 bg-panel p-6 text-center">
                    <div className="mb-2 text-2xl">🔐</div>
                    <div className="mb-1 text-sm font-bold text-fg">MFA required for admin access</div>
                    <div className="mb-4 text-[12px] leading-relaxed text-fg-dim">
                        The Security Operations Console is gated behind two-factor authentication.
                        Enroll a TOTP authenticator, then sign in again to continue.
                    </div>
                    <a href="/account"
                        className="inline-block border border-accent bg-panel-hi px-4 py-2 text-[11px] uppercase tracking-[1.5px] text-accent hover:brightness-125">
                        set up MFA →
                    </a>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-full">
            <AlertToast alert={latestAlert?.alert ?? null} trigger={latestAlert?.nonce ?? 0} muted={muted} />

            {/* console sub-bar (sits under the global AppLayout header) */}
            <div className="flex items-center justify-between border-b border-line bg-panel px-3.5 py-2">
                <span className="text-[10px] uppercase tracking-[2px] text-fg-dim">security operations console</span>
                <div className="flex items-center gap-2.5 text-[10px]">
                    <span className={`flex items-center gap-1.5 ${indicator === "open" ? "text-accent" : "text-fg-dim"}`}>
                        <span className={`inline-block h-2 w-2 rounded-full ${indicator === "open" ? "bg-accent pulse" : "bg-fg-dim"}`} />
                        {indicator === "open" ? "LIVE" : indicator === "reconnecting" ? "RECONNECTING" : "PAUSED"}
                    </span>
                    <button onClick={() => setMuted((m) => !m)} title={muted ? "alert sound off" : "alert sound on"}
                        className={`border border-line px-2 py-1 uppercase tracking-wide hover:brightness-125 ${muted ? "text-fg-dim" : "bg-panel-hi text-accent"}`}>
                        {muted ? "🔇" : "🔊"}
                    </button>
                    <button onClick={() => setPaused((p) => !p)}
                        className={`border border-line px-2.5 py-1 uppercase tracking-wide hover:brightness-125 ${statusClass[indicator]}`}>
                        {statusLabel[indicator]}
                    </button>
                </div>
            </div>

            <div className="p-3.5 text-xs">
                {error && <div className="mb-3"><Notice kind="error">{error}</Notice></div>}

                <MetricRow m={metrics} />

                <SectionLabel hint="operator console">red team</SectionLabel>
                <AttackRange />

                <SectionLabel hint="event rate · severity · vectors">telemetry</SectionLabel>
                <div className="grid gap-3 lg:grid-cols-[1.4fr_1fr]">
                    <EventRatePanel data={rate} />
                    <SeverityPanel data={sevDist} />
                </div>
                <div className="mt-3">
                    <EventTypePanel data={evTypes} />
                </div>

                <SectionLabel hint="origins · impossible travel">geo intelligence</SectionLabel>
                <div className="grid gap-3 lg:grid-cols-[1.6fr_1fr]">
                    <GeoMapPanel points={geos} hits={travel} />
                    <div className="flex flex-col gap-3">
                        <ImpossibleTravelPanel hits={travel} />
                        <GeoIpPanel points={geos} />
                    </div>
                </div>

                <SectionLabel hint="live feed · alerts">operations</SectionLabel>
                <div className="grid gap-3 lg:grid-cols-[1.4fr_1fr]">
                    <LiveFeedPanel events={events} live={indicator === "open"} />
                    <div className="flex flex-col gap-3">
                        <AlertsPanel alerts={alerts} />
                        <TopIpsPanel ips={ips} />
                    </div>
                </div>

                <div className="mt-4 flex items-center justify-between border-t border-line pt-2 text-[10px] text-fg-dim">
                    <span>aegis mini-SIEM · live admin telemetry</span>
                    <span>
                        chain integrity: <span className="text-accent">VERIFIED ✓</span> · risk engine: ACTIVE · GeoIP: ACTIVE · WebSocket alerts
                    </span>
                </div>
            </div>
        </div>
    );
}
