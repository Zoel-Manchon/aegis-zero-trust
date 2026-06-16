import { Bar, BarChart, Cell, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Panel } from "@/components/Panel";
import type { EventTypeCount } from "@/features/security/derive";

const tooltipStyle = {
    background: "#0c1016",
    border: "1px solid #1b2430",
    fontSize: 11,
    fontFamily: "var(--font-mono)",
} as const;

/* Horizontal breakdown of which event types are firing — populates as you run
 * attacks, so the new scenarios (fingerprint spoof, session hijack, …) show up
 * here distinctly. Hostile/attack-indicating types are drawn in the warn color. */
export function EventTypePanel({ data }: { data: EventTypeCount[] }) {
    return (
        <Panel title="attack vectors — event types" right="by volume">
            {data.length === 0 ? (
                <p className="px-1 py-8 text-center text-[11px] text-fg-dim">
                    no events yet — launch an attack to populate
                </p>
            ) : (
                <ResponsiveContainer width="100%" height={Math.max(130, data.length * 28)}>
                    <BarChart data={data} layout="vertical" margin={{ top: 4, right: 16, left: 4, bottom: 0 }}>
                        <XAxis type="number" tick={{ fill: "#5d6b7d", fontSize: 9 }} allowDecimals={false} />
                        <YAxis
                            type="category"
                            dataKey="label"
                            width={150}
                            tick={{ fill: "#8a99ab", fontSize: 9 }}
                        />
                        <Tooltip contentStyle={tooltipStyle} cursor={{ fill: "rgba(255,255,255,0.03)" }} />
                        <Bar dataKey="count" radius={[0, 2, 2, 0]}>
                            {data.map((d) => (
                                <Cell key={d.type} fill={d.hostile ? "#ff8a3d" : "#39ff8b"} />
                            ))}
                        </Bar>
                    </BarChart>
                </ResponsiveContainer>
            )}
        </Panel>
    );
}
