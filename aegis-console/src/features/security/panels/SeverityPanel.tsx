import { Bar, BarChart, CartesianGrid, Cell, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Panel } from "@/components/Panel";
import { sevHex } from "@/components/severity";
import type { SevSlice } from "@/features/security/derive";

const tooltipStyle = {
    background: "#0c1016",
    border: "1px solid #1b2430",
    fontSize: 11,
    fontFamily: "var(--font-mono)",
} as const;

export function SeverityPanel({ data }: { data: SevSlice[] }) {
    return (
        <Panel title="severity distribution">
            <ResponsiveContainer width="100%" height={170}>
                <BarChart data={data} margin={{ top: 6, right: 6, left: -22, bottom: 0 }}>
                    <CartesianGrid stroke="#141b24" vertical={false} />
                    <XAxis dataKey="sev" tick={{ fill: "#5d6b7d", fontSize: 9 }} />
                    <YAxis tick={{ fill: "#5d6b7d", fontSize: 9 }} allowDecimals={false} />
                    <Tooltip contentStyle={tooltipStyle} cursor={{ fill: "rgba(255,255,255,0.03)" }} />
                    <Bar dataKey="count">
                        {data.map((d) => (
                            <Cell key={d.sev} fill={sevHex[d.sev]} />
                        ))}
                    </Bar>
                </BarChart>
            </ResponsiveContainer>
        </Panel>
    );
}
