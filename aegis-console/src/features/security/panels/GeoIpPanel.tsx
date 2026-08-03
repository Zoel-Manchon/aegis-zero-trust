import { Panel } from "@/components/Panel";
import type { GeoPoint } from "@/features/security/derive";

/* GeoIP origins. Network type is the column that matters most: a datacenter or
 * hosting ASN behind a "user" login is a finding on its own. */
export function GeoIpPanel({ points, className = "" }: { points: GeoPoint[]; className?: string }) {
    const max = Math.max(1, ...points.map((p) => p.count));
    return (
        <Panel title="GeoIP origins" right="network type" className={className} bodyClassName="">
            {points.length === 0 ? (
                <div className="p-3 text-[11px] text-fg-dim">
                    No GeoIP metadata yet. Sign in — or launch a scenario — from a different origin.
                </div>
            ) : (
                <table className="table text-[11px]">
                    <tbody>
                        {points.map((p) => (
                            <tr key={`${p.city}-${p.country}`}>
                                <td className="px-3 py-1.5">
                                    {p.city}, {p.country}
                                </td>
                                <td className="px-1.5 py-1.5 text-fg-dim">{p.network_type}</td>
                                <td
                                    className={`px-3 py-1.5 text-right ${
                                        p.count / max > 0.6 ? "text-accent" : "text-neutral-800"
                                    }`}
                                >
                                    {p.count}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </Panel>
    );
}
