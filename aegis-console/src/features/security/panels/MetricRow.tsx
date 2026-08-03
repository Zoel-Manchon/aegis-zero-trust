import { Stat } from "@/components/Stat";
import type { SecurityMetrics } from "@/types";

/* The KPI strip. One bordered block split into seven cells rather than seven
 * floating cards: these numbers are a single instrument panel, and hairlines
 * between them read faster than gaps around them. */
export function MetricRow({ m }: { m: SecurityMetrics | null }) {
    const v = (n?: number) => (n ?? 0).toLocaleString();
    return (
        <div className="grid grid-cols-2 border border-line bg-panel sm:grid-cols-4 lg:grid-cols-7">
            <Stat label="Total events" value={v(m?.total_events)} sub="rolling window" />
            <Stat label="Critical" value={v(m?.critical_events)} accent="text-accent" sub="escalate now" />
            <Stat label="High" value={v(m?.high_events)} accent="text-accent-600" sub="triage queue" />
            <Stat
                label="Refresh replays"
                value={v(m?.refresh_replays)}
                accent={m?.refresh_replays ? "text-accent" : "text-fg"}
                sub="token theft signal"
            />
            <Stat label="Policy denials" value={v(m?.policy_denials)} sub="zero-trust engine" />
            <Stat label="MFA failures" value={v(m?.mfa_failures)} sub="second factor" />
            <Stat label="BF lockouts" value={v(m?.brute_force_lockouts)} sub="auto-mitigated" />
        </div>
    );
}
