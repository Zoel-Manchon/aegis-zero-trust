import { Panel } from "@/components/Panel";
import { sevChip, sevHex } from "@/components/severity";
import type { SecurityAlert } from "@/types";

/* Correlated alerts. Each row is railed on the left in its severity colour —
 * the rail is the only thing you need to scan to triage the list. */
export function AlertsPanel({ alerts, className = "" }: { alerts: SecurityAlert[]; className?: string }) {
    return (
        <Panel
            title="Correlated alerts"
            right={`${alerts.length} open`}
            className={className}
            bodyClassName=""
        >
            <div className="max-h-[220px] overflow-y-auto">
                {alerts.length === 0 ? (
                    <div className="p-3 text-[11px] text-fg-dim">
                        No correlated alerts in window. System nominal.
                    </div>
                ) : (
                    alerts.map((a, i) => (
                        <div
                            key={`${a.alert_type}-${i}`}
                            className="border-b border-neutral-200 border-l-[3px] px-3 py-2.5"
                            style={{ borderLeftColor: sevHex[a.severity] }}
                        >
                            <div className="flex items-baseline justify-between gap-2">
                                <span className="font-heading text-[12px] font-extrabold">{a.title}</span>
                                <span
                                    className={`shrink-0 px-1.5 py-0.5 text-[9px] uppercase tracking-[0.14em] ${sevChip[a.severity]}`}
                                >
                                    {a.severity}
                                </span>
                            </div>
                            <div className="mt-0.5 text-[11px] leading-[1.4] text-fg-dim">
                                {a.description}
                            </div>
                        </div>
                    ))
                )}
            </div>
        </Panel>
    );
}
