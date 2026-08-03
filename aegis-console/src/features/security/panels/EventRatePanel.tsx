import { Panel } from "@/components/Panel";
import { RATE_BUCKETS, type TimeBucket } from "@/features/security/derive";

const H = 140; // px of bar area, matching the 168px track minus padding

/* Event rate. Deliberately not a smoothed area chart: each bar is one 30s
 * interval, and the high/critical share is stacked on top in accent, so the
 * question "is the hostile share growing?" is answered by the shape itself. */
export function EventRatePanel({ data }: { data: TimeBucket[] }) {
    const peak = Math.max(1, ...data.map((b) => b.count));

    return (
        <Panel
            title="Event rate"
            right={`last ${RATE_BUCKETS} intervals · peak ${peak}/int`}
            bodyClassName=""
        >
            <div className="flex h-[168px] items-end gap-0.5 p-3">
                {data.map((b) => {
                    const crit = Math.round((b.crit / peak) * H);
                    const rest = Math.round(((b.count - b.crit) / peak) * H);
                    return (
                        <div
                            key={b.t}
                            className="flex h-full flex-1 flex-col justify-end"
                            title={`${b.count} events · ${b.crit} high/critical`}
                        >
                            <div className="bg-accent" style={{ height: crit }} />
                            <div className="bg-neutral-400" style={{ height: rest }} />
                        </div>
                    );
                })}
            </div>
            <div className="flex items-center justify-between border-t border-line px-3 py-1.5 text-[10px] uppercase tracking-[0.12em] text-fg-dim">
                <span>−20 min</span>
                <span className="flex gap-3.5">
                    <span className="text-accent-700">■ high / critical</span>
                    <span>■ all events</span>
                </span>
                <span>now</span>
            </div>
        </Panel>
    );
}
