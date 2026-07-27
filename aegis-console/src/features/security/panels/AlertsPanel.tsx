import { Panel } from "@/components/Panel";
import { SevDot } from "@/components/SevDot";
import { sevText } from "@/components/severity";
import type { SecurityAlert } from "@/types";

export function AlertsPanel({ alerts }: { alerts: SecurityAlert[] }) {
    return (
        <Panel title="active alerts" right={`${alerts.length} firing`}>
            <div className="min-h-[150px]">
                {alerts.length === 0 ? (
                    <div className="p-2.5 text-[11px] text-fg-dim">
                        No correlated alerts in window. System nominal.
                    </div>
                ) : (
                    alerts.map((a, i) => (
                        <div key={`${a.alert_type}-${i}`}
                            className="flex items-start gap-2 border-b border-grid px-1.5 py-2">
                            <SevDot sev={a.severity} />
                            <div className="min-w-0">
                                <div className={`font-semibold ${sevText[a.severity]}`}>{a.title}</div>
                                <div className="text-[10px] text-fg-dim">{a.description}</div>
                            </div>
                            <div className={`ml-auto font-bold ${sevText[a.severity]}`}>{a.count}</div>
                        </div>
                    ))
                )}
            </div>
        </Panel>
    );
}
