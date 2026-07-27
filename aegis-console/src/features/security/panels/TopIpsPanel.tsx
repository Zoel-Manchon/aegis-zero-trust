import { Panel } from "@/components/Panel";
import type { IpCount } from "@/features/security/derive";

export function TopIpsPanel({ ips }: { ips: IpCount[] }) {
    const max = ips[0]?.count ?? 1;
    return (
        <Panel title="top source ips" right={ips.length ? `${ips.length} sources` : undefined}>
            <div>
                {ips.length === 0 && (
                    <div className="p-2 text-[11px] text-fg-dim">No source addresses seen yet.</div>
                )}
                {ips.map((row, i) => (
                    <div key={row.ip} className="flex items-center gap-2 px-1.5 py-1.5 text-[11px]">
                        <span className="w-4 text-fg-mute tabular-nums">{i + 1}</span>
                        <span className="w-[120px] truncate text-fg">{row.ip}</span>
                        <div className="h-2 flex-1 bg-grid">
                            <div
                                className="h-full bg-accent"
                                style={{ width: `${(row.count / max) * 100}%`, opacity: 0.85 }}
                            />
                        </div>
                        <span className="w-7 text-right tabular-nums text-fg-dim">{row.count}</span>
                    </div>
                ))}
            </div>
        </Panel>
    );
}
