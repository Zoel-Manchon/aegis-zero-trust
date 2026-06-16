import { Panel } from "@/components/Panel";
import type { ImpossibleTravelHit } from "@/features/security/derive";

export function ImpossibleTravelPanel({ hits }: { hits: ImpossibleTravelHit[] }) {
    return (
        <Panel title="impossible travel" right={`${hits.length} hits`}>
            <div className="min-h-[120px] space-y-1.5">
                {hits.length === 0 ? (
                    <div className="p-2.5 text-[11px] text-fg-dim">No impossible-travel violations in the current event window.</div>
                ) : hits.map((h) => (
                    <div key={h.id} className="border border-sev-critical/30 bg-sev-critical/5 px-2 py-1.5">
                        <div className="text-[11px] font-bold text-sev-critical">user {h.user_id ?? "unknown"} · {Math.round(h.speed_kmh)} km/h</div>
                        <div className="text-[10px] text-fg-dim">{h.from} → {h.to} · {Math.round(h.distance_km)} km · {h.ip}</div>
                    </div>
                ))}
            </div>
        </Panel>
    );
}
