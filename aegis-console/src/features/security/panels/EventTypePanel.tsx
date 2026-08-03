import { Panel } from "@/components/Panel";
import type { EventTypeCount } from "@/features/security/derive";

/* Attack vectors — which event types are firing. Two columns of meters, hostile
 * types in vermilion, benign in ink: the red rows are the ones to read. */
export function EventTypePanel({ data }: { data: EventTypeCount[] }) {
    const max = Math.max(1, ...data.map((d) => d.count));
    return (
        <Panel title="Attack vectors" right="top event types by volume" bodyClassName="p-3">
            {data.length === 0 ? (
                <p className="py-8 text-center text-[11px] text-fg-dim">
                    No events yet. Launch a scenario from the attack range to populate this panel.
                </p>
            ) : (
                <div className="grid gap-x-6 md:grid-cols-2">
                    {data.map((d) => (
                        <div
                            key={d.type}
                            className="grid grid-cols-[minmax(0,210px)_1fr_44px] items-center gap-2.5 border-b border-neutral-200 py-1.5"
                        >
                            <span
                                className={`truncate text-[11px] uppercase tracking-[0.06em] ${
                                    d.hostile ? "text-accent-600" : "text-fg-dim"
                                }`}
                            >
                                {d.label}
                            </span>
                            <span className="h-2.5 bg-neutral-200">
                                <span
                                    className={`block h-full ${d.hostile ? "bg-accent-600" : "bg-neutral-700"}`}
                                    style={{ width: `${Math.round((d.count / max) * 100)}%` }}
                                />
                            </span>
                            <span className="text-right text-[11px] text-fg-dim">{d.count}</span>
                        </div>
                    ))}
                </div>
            )}
        </Panel>
    );
}
