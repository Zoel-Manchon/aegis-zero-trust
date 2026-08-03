import { Panel } from "@/components/Panel";
import type { ImpossibleTravelHit } from "@/features/security/derive";

/* Impossible travel — the highest-signal detection in the system, so it gets
 * its own panel, an accent title, and a tinted row for every hit. */
export function ImpossibleTravelPanel({ hits }: { hits: ImpossibleTravelHit[] }) {
    return (
        <Panel
            title={<span className="text-accent-700">Impossible travel</span>}
            right={`${hits.length} hits`}
            bodyClassName=""
        >
            <div className="max-h-[148px] overflow-y-auto">
                {hits.length === 0 ? (
                    <div className="p-3 text-[11px] text-fg-dim">
                        No impossible-travel signals in window.
                    </div>
                ) : (
                    hits.map((h) => (
                        <div
                            key={h.id}
                            className="border-b border-neutral-200 bg-accent-100 px-3 py-2"
                        >
                            <div className="font-heading text-[11px] font-extrabold tracking-[0.04em]">
                                {h.from} → {h.to}
                            </div>
                            <div className="mt-0.5 text-[10px] uppercase tracking-[0.08em] text-fg-dim">
                                uid {h.user_id ?? "—"} · {Math.round(h.distance_km).toLocaleString()} km ·{" "}
                                {Math.round(h.speed_kmh).toLocaleString()} km/h ·{" "}
                                {new Date(h.created_at).toISOString().slice(11, 19)}
                            </div>
                        </div>
                    ))
                )}
            </div>
        </Panel>
    );
}
