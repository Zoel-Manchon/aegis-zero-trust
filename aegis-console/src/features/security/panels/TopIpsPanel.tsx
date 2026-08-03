import { Panel } from "@/components/Panel";
import type { IpCount } from "@/features/security/derive";

/* Top talkers. An address that owns more than ~70% of the loudest bar turns
 * vermilion — that's the one worth blocking. */
export function TopIpsPanel({ ips, className = "" }: { ips: IpCount[]; className?: string }) {
    const max = ips[0]?.count ?? 1;
    return (
        <Panel title="Top talkers" right="source IP" className={className} bodyClassName="px-3 py-2.5">
            {ips.length === 0 ? (
                <div className="text-[11px] text-fg-dim">No source addresses seen yet.</div>
            ) : (
                <div className="flex flex-col gap-1.5">
                    {ips.map((row) => {
                        const pct = Math.round((row.count / max) * 100);
                        return (
                            <div
                                key={row.ip}
                                className="grid grid-cols-[118px_1fr_32px] items-center gap-2.5"
                            >
                                <span className="truncate text-[11px]">{row.ip}</span>
                                <span className="h-2.5 bg-neutral-200">
                                    <span
                                        className={`block h-full ${pct > 70 ? "bg-accent" : "bg-neutral-600"}`}
                                        style={{ width: `${pct}%` }}
                                    />
                                </span>
                                <span className="text-right text-[11px] text-fg-dim">{row.count}</span>
                            </div>
                        );
                    })}
                </div>
            )}
        </Panel>
    );
}
