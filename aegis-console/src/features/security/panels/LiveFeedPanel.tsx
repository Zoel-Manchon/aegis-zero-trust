import { useMemo, useState } from "react";
import { Panel } from "@/components/Panel";
import { SevDot } from "@/components/SevDot";
import { SEVERITIES, sevText } from "@/components/severity";
import type { Severity, SecurityEvent } from "@/types";

/** "City, CC" from event geoip metadata, if present. */
function geoLabel(e: SecurityEvent): string | null {
    const g = e.metadata?.geoip as Record<string, unknown> | undefined;
    if (!g) return null;
    const city = g.city ? String(g.city) : null;
    const country = g.country ? String(g.country) : null;
    const label = [city, country].filter(Boolean).join(", ");
    return label.length > 0 ? label : null;
}

export function LiveFeedPanel({ events, live }: { events: SecurityEvent[]; live: boolean }) {
    const [filter, setFilter] = useState<"all" | Severity>("all");
    const shown = filter === "all" ? events : events.filter((e) => e.severity === filter);

    const counts = useMemo(() => {
        let crit = 0;
        let high = 0;
        for (const e of events) {
            if (e.severity === "critical") crit++;
            else if (e.severity === "high") high++;
        }
        return { total: events.length, crit, high };
    }, [events]);

    return (
        <Panel
            title="live event feed · SOC reporter"
            right={
                <div className="flex gap-1">
                    {(["all", ...SEVERITIES] as const).map((s) => (
                        <button key={s} onClick={() => setFilter(s)}
                            className={`border px-2 py-0.5 text-[9px] uppercase tracking-wide hover:brightness-125 ${
                                filter === s
                                    ? `border-line bg-panel-hi ${s === "all" ? "text-accent" : sevText[s as Severity]}`
                                    : "border-transparent text-fg-dim"
                            }`}>
                            {s}
                        </button>
                    ))}
                </div>
            }
        >
            <div className="flex items-center gap-3 border-b border-grid bg-panel-hi/40 px-2 py-1 text-[9px] uppercase tracking-[1.5px] text-fg-dim">
                <span>{counts.total} events</span>
                <span className="text-sev-critical">{counts.crit} crit</span>
                <span className="text-sev-high">{counts.high} high</span>
                <span className={`ml-auto flex items-center gap-1 ${live ? "text-accent" : "text-fg-dim"}`}>
                    <span className={`inline-block h-1.5 w-1.5 rounded-full ${live ? "bg-accent pulse" : "bg-fg-dim"}`} />
                    {live ? "streaming" : "paused"}
                </span>
            </div>

            <div className="h-[320px] overflow-y-auto font-mono text-[11px]">
                {shown.length === 0 ? (
                    <div className="p-2.5 text-fg-dim">No events match this filter.</div>
                ) : (
                    <table className="w-full border-collapse">
                        <tbody>
                            {shown.slice(0, 150).map((e, i) => {
                                const loc = geoLabel(e);
                                const crit = e.severity === "critical";
                                return (
                                    <tr key={e.id}
                                        className={`border-b border-grid ${i === 0 && live ? "row-flash" : ""} ${crit ? "bg-sev-critical/[0.06]" : ""}`}>
                                        <td className="whitespace-nowrap px-1.5 py-1.5 text-fg-dim">
                                            {new Date(e.created_at).toISOString().slice(11, 19)}
                                        </td>
                                        <td className="w-4 px-1 py-1.5"><SevDot sev={e.severity} /></td>
                                        <td className={`whitespace-nowrap px-1.5 py-1.5 font-semibold ${sevText[e.severity]}`}>
                                            {e.event_type}
                                        </td>
                                        <td className="px-1.5 py-1.5 text-fg-dim">{e.user_id ? `uid:${e.user_id}` : "—"}</td>
                                        <td className="px-1.5 py-1.5 text-fg-dim">{e.ip_address ?? "—"}</td>
                                        <td className="whitespace-nowrap px-1.5 py-1.5 text-accent/80">{loc ?? ""}</td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                )}
            </div>
        </Panel>
    );
}
