import { Panel } from "@/components/Panel";
import type { IpCount } from "@/features/security/derive";

export function TopIpsPanel({ ips }: { ips: IpCount[] }) {
    const max = ips[0]?.count ?? 1;
    return (
        <Panel title="top source ips">
            <div>
                {ips.length === 0 && <div className="p-1.5 text-[11px] text-fg-dim">no data</div>}
                {ips.map((row, i) => (
                    <div key={row.ip} className="flex items-center gap-2 px-1.5 py-1 text-[11px]">
                        <span className="w-3.5 text-fg-dim">{i + 1}</span>
                        <span className="w-[110px] truncate">{row.ip}</span>
                        <div className="h-1.5 flex-1 bg-grid">
                            <div className="h-full bg-accent/60" style={{ width: `${(row.count / max) * 100}%` }} />
                        </div>
                        <span className="w-6 text-right text-fg-dim">{row.count}</span>
                    </div>
                ))}
            </div>
        </Panel>
    );
}
