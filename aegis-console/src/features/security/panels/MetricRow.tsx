import { Stat } from "@/components/Stat";
import type { SecurityMetrics } from "@/types";

export function MetricRow({ m }: { m: SecurityMetrics | null }) {
    const v = (n?: number) => (n ?? 0).toLocaleString();
    return (
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4 lg:grid-cols-7">
            <Stat label="Total Events" value={v(m?.total_events)} />
            <Stat label="Critical" value={v(m?.critical_events)} accent="text-sev-critical" />
            <Stat label="High" value={v(m?.high_events)} accent="text-sev-high" />
            <Stat label="Refresh Replays" value={v(m?.refresh_replays)}
                accent={m?.refresh_replays ? "text-sev-critical" : "text-fg"} sub="token theft signal" />
            <Stat label="Policy Denials" value={v(m?.policy_denials)} accent="text-sev-medium" />
            <Stat label="MFA Failures" value={v(m?.mfa_failures)} accent="text-sev-medium" />
            <Stat label="BF Lockouts" value={v(m?.brute_force_lockouts)} accent="text-sev-high" />
        </div>
    );
}
