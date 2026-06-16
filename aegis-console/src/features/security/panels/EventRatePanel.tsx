import { Area, AreaChart, CartesianGrid, Line, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Panel } from "@/components/Panel";
import type { TimeBucket } from "@/features/security/derive";

const tooltipStyle = {
    background: "#0c1016",
    border: "1px solid #1b2430",
    fontSize: 11,
    fontFamily: "var(--font-mono)",
} as const;

export function EventRatePanel({ data }: { data: TimeBucket[] }) {
    return (
        <Panel title="event rate — last 30 min" right="events/min">
            <ResponsiveContainer width="100%" height={170}>
                <AreaChart data={data} margin={{ top: 6, right: 6, left: -22, bottom: 0 }}>
                    <defs>
                        <linearGradient id="evrate" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stopColor="#39ff8b" stopOpacity={0.5} />
                            <stop offset="100%" stopColor="#39ff8b" stopOpacity={0} />
                        </linearGradient>
                    </defs>
                    <CartesianGrid stroke="#141b24" vertical={false} />
                    <XAxis dataKey="t" tick={{ fill: "#5d6b7d", fontSize: 9 }}
                        tickFormatter={(value: number) => `-${29 - value}m`} interval={5} />
                    <YAxis tick={{ fill: "#5d6b7d", fontSize: 9 }} allowDecimals={false} />
                    <Tooltip contentStyle={tooltipStyle} />
                    <Area type="monotone" dataKey="count" stroke="#39ff8b" strokeWidth={1.5} fill="url(#evrate)" />
                    <Line type="monotone" dataKey="crit" stroke="#ff3b3b" strokeWidth={1.5} dot={false} />
                </AreaChart>
            </ResponsiveContainer>
        </Panel>
    );
}
