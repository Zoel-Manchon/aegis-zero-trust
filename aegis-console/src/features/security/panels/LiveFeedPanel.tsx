import { useMemo, useState } from "react";
import { Panel } from "@/components/Panel";
import { SevDot } from "@/components/SevDot";
import { SEVERITIES, sevChip } from "@/components/severity";
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

/* The live feed — the panel the console lives on. Fixed column widths and
 * tabular figures so rows don't reflow as events land; the newest row flashes
 * once in accent tint, which is the only motion in the layout. */
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
            title="Live event feed"
            right={
                <span className="flex gap-0.5">
                    {(["all", ...SEVERITIES] as const).map((s) => {
                        const active = filter === s;
                        const chip =
                            s === "all" ? "bg-fg text-bg" : sevChip[s as Severity];
                        return (
                            <button
                                key={s}
                                onClick={() => setFilter(s)}
                                className={`cursor-pointer px-1.5 py-1 text-[9px] uppercase tracking-[0.12em] ${
                                    active
                                        ? `border border-transparent ${chip}`
                                        : "border border-neutral-300 bg-transparent text-fg-dim hover:text-fg"
                                }`}
                            >
                                {s}
                            </button>
                        );
                    })}
                </span>
            }
            bodyClassName=""
        >
            <div className="flex gap-4 border-b border-neutral-300 px-3 py-1.5 text-[10px] uppercase tracking-[0.14em] text-fg-dim">
                <span>{counts.total.toLocaleString()} events</span>
                <span className="text-accent-700">{counts.crit} crit</span>
                <span>{counts.high} high</span>
                <span className="ml-auto">{live ? "Live" : "Paused"}</span>
            </div>

            <div className="h-[340px] overflow-y-auto overflow-x-hidden">
                {shown.length === 0 ? (
                    <div className="p-3 text-[11px] text-fg-dim">No events match this filter.</div>
                ) : (
                    <table className="table table-fixed text-[11px]">
                        <tbody>
                            {shown.slice(0, 150).map((e, i) => {
                                const loc = geoLabel(e);
                                const hot = e.severity === "critical" || e.severity === "high";
                                return (
                                    <tr
                                        key={e.id}
                                        className={`${i === 0 && live ? "row-flash" : ""} ${
                                            e.severity === "critical" ? "bg-accent-100" : ""
                                        }`}
                                    >
                                        <td className="w-[74px] whitespace-nowrap px-2.5 py-1 text-fg-dim">
                                            {new Date(e.created_at).toISOString().slice(11, 19)}
                                        </td>
                                        <td className="w-3.5 py-1">
                                            <SevDot sev={e.severity} />
                                        </td>
                                        <td
                                            className={`truncate px-2 py-1 font-heading text-[10px] font-extrabold tracking-[0.06em] ${
                                                hot ? "text-accent-700" : "text-fg"
                                            }`}
                                        >
                                            {e.event_type}
                                        </td>
                                        <td className="w-[62px] whitespace-nowrap px-2 py-1 text-fg-dim">
                                            {e.user_id ? `uid:${e.user_id}` : "—"}
                                        </td>
                                        <td className="w-[116px] whitespace-nowrap px-2 py-1 text-fg-dim">
                                            {e.ip_address ?? "—"}
                                        </td>
                                        <td className="w-[124px] truncate px-2.5 py-1 text-right text-neutral-800">
                                            {loc ?? ""}
                                        </td>
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
