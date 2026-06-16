import { Panel } from "@/components/Panel";
import type { GeoPoint } from "@/features/security/derive";

export function GeoIpPanel({ points }: { points: GeoPoint[] }) {
    return (
        <Panel title="geo ip intelligence" right={`${points.length} locations`}>
            <div className="min-h-[140px] space-y-1.5">
                {points.length === 0 ? (
                    <div className="p-2.5 text-[11px] text-fg-dim">No GeoIP metadata yet. Generate logins from different IPs.</div>
                ) : points.map((p) => (
                    <div key={`${p.city}-${p.country}`} className="grid grid-cols-[1fr_auto] gap-2 border-b border-grid px-1.5 py-1.5">
                        <div className="min-w-0">
                            <div className="truncate text-[11px] text-fg">{p.city}, {p.country}</div>
                            <div className="truncate text-[10px] text-fg-dim">{p.ip} · {p.network_type} · {p.latitude.toFixed(2)}, {p.longitude.toFixed(2)}</div>
                        </div>
                        <div className="font-bold text-accent">{p.count}</div>
                    </div>
                ))}
            </div>
        </Panel>
    );
}
