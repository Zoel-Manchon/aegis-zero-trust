import { Panel } from "@/components/Panel";
import { sevHex } from "@/components/severity";
import type { SevSlice } from "@/features/security/derive";

/* Severity distribution as five stacked meters. A bar per level beats a column
 * chart here because the label, the count and the share all sit on one line. */
export function SeverityPanel({ data }: { data: SevSlice[] }) {
    const total = data.reduce((n, d) => n + d.count, 0);
    return (
        <Panel title="Severity distribution" right={`${total.toLocaleString()} events`} bodyClassName="p-3">
            <div className="flex flex-col gap-2.5">
                {data.map((d) => {
                    const pct = total ? Math.round((d.count / total) * 100) : 0;
                    return (
                        <div key={d.sev}>
                            <div className="mb-1 flex justify-between text-[11px] uppercase tracking-[0.12em]">
                                <span>{d.sev}</span>
                                <span className="text-fg-dim">
                                    {d.count.toLocaleString()} · {pct}%
                                </span>
                            </div>
                            <div className="h-3 bg-neutral-200">
                                <div
                                    className="h-full"
                                    style={{ width: `${pct}%`, background: sevHex[d.sev] }}
                                />
                            </div>
                        </div>
                    );
                })}
            </div>
        </Panel>
    );
}
