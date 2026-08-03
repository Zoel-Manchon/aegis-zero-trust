/* =============================================================================
 * aegis :: Security Operations Console
 *
 * Three labeled bands, in the order an operator reads them:
 *   KPI strip   — is anything on fire right now?
 *   TELEMETRY   — event rate, severity mix, attack vectors
 *   GEO         — where it's coming from + impossible travel
 *   OPERATIONS  — the live feed, correlated alerts, top talkers
 *
 * The red team console lives on its own screen (/range) so this one stays a
 * pure read surface. Data engine: useSecurityData (push events/alerts, polled
 * metrics). Admin-gated upstream by <RoleRoute require="admin">.
 * ========================================================================== */

import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Notice } from "@/components/Notice";
import { SectionLabel } from "@/components/SectionLabel";
import { SubBar } from "@/app/SubBar";
import { useSecurityData } from "@/features/security/useSecurityData";
import {
    eventTypeDistribution,
    geoPoints,
    impossibleTravelHits,
    severityDistribution,
    timeline,
    topIps,
} from "@/features/security/derive";
import { MetricRow } from "@/features/security/panels/MetricRow";
import { EventRatePanel } from "@/features/security/panels/EventRatePanel";
import { SeverityPanel } from "@/features/security/panels/SeverityPanel";
import { EventTypePanel } from "@/features/security/panels/EventTypePanel";
import { LiveFeedPanel } from "@/features/security/panels/LiveFeedPanel";
import { AlertsPanel } from "@/features/security/panels/AlertsPanel";
import { TopIpsPanel } from "@/features/security/panels/TopIpsPanel";
import { GeoIpPanel } from "@/features/security/panels/GeoIpPanel";
import { ImpossibleTravelPanel } from "@/features/security/panels/ImpossibleTravelPanel";
import { AlertToast } from "@/components/AlertToast";
import { GeoMapPanel } from "@/features/security/GeoMapPanel";

export default function Dashboard() {
    const { metrics, events, alerts, latestAlert, status, paused, setPaused, error, mfaRequired } =
        useSecurityData();
    const [muted, setMuted] = useState(false);

    const rate = useMemo(() => timeline(events), [events]);
    const sevDist = useMemo(() => severityDistribution(events), [events]);
    const evTypes = useMemo(() => eventTypeDistribution(events), [events]);
    const ips = useMemo(() => topIps(events), [events]);
    const geos = useMemo(() => geoPoints(events), [events]);
    const travel = useMemo(() => impossibleTravelHits(events), [events]);

    if (mfaRequired) {
        return (
            <>
                <SubBar label="security operations console" />
                <div className="flex min-h-[60vh] items-center justify-center p-6">
                    <div className="w-full max-w-md border border-line bg-panel">
                        <div className="micro border-b-2 border-line px-3 py-2 text-accent-700">
                            Second factor required
                        </div>
                        <div className="space-y-4 p-5">
                            <p className="text-[13px] leading-[1.6] text-neutral-800">
                                The operations console is gated behind two-factor authentication.
                                Enroll a TOTP authenticator, then sign in again to continue.
                            </p>
                            <Link to="/account" className="btn btn-primary text-[12px] uppercase tracking-[0.1em]">
                                Set up MFA →
                            </Link>
                        </div>
                    </div>
                </div>
            </>
        );
    }

    return (
        <>
            <AlertToast alert={latestAlert?.alert ?? null} trigger={latestAlert?.nonce ?? 0} muted={muted} />

            <SubBar
                label="security operations console"
                stream={{
                    live: status === "open",
                    reconnecting: status === "reconnecting",
                    paused,
                    onTogglePause: () => setPaused((p) => !p),
                    muted,
                    onToggleMute: () => setMuted((m) => !m),
                }}
            />

            <div className="p-4">
                {error && (
                    <div className="mb-3">
                        <Notice kind="error">{error}</Notice>
                    </div>
                )}

                <MetricRow m={metrics} />

                <SectionLabel hint="Event rate · severity · vectors">Telemetry</SectionLabel>
                <div className="mt-3 grid gap-3 lg:grid-cols-[1.5fr_1fr]">
                    <EventRatePanel data={rate} />
                    <SeverityPanel data={sevDist} />
                </div>
                <div className="mt-3">
                    <EventTypePanel data={evTypes} />
                </div>

                <SectionLabel hint="Origins · impossible travel">Geo intelligence</SectionLabel>
                <div className="mt-3 grid gap-3 lg:grid-cols-[1.6fr_1fr]">
                    <GeoMapPanel points={geos} hits={travel} />
                    <div className="flex flex-col gap-3">
                        <ImpossibleTravelPanel hits={travel} />
                        <GeoIpPanel points={geos} className="flex-1" />
                    </div>
                </div>

                <SectionLabel hint="Live feed · alerts · top talkers">Operations</SectionLabel>
                <div className="mt-3 grid gap-3 lg:grid-cols-[1.5fr_1fr]">
                    <LiveFeedPanel events={events} live={status === "open" && !paused} />
                    <div className="flex flex-col gap-3">
                        <AlertsPanel alerts={alerts} />
                        <TopIpsPanel ips={ips} className="flex-1" />
                    </div>
                </div>

                <div className="mt-5 flex flex-wrap justify-between gap-2 border-t-2 border-line pt-2 text-[10px] uppercase tracking-[0.12em] text-fg-dim">
                    <span>Aegis mini-SIEM · live admin telemetry</span>
                    <span>
                        Chain integrity <span className="text-accent-700">verified</span> · risk engine
                        active · GeoIP active · WebSocket alerts
                    </span>
                </div>
            </div>
        </>
    );
}
