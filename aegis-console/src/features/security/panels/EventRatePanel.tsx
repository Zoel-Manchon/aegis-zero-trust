import { Area, AreaChart, CartesianGrid, Line, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Panel } from "@/components/Panel";
import { axisTick, chart, tooltipLineCursor } from "@/features/security/chartTheme";
import type { TimeBucket } from "@/features/security/derive";

export function EventRatePanel({ data }: { data: TimeBucket[] }) {
    return (
        <Panel title="event rate — last 30 min" right="events/min">
            <ResponsiveContainer width="100%" height={180}>
                <AreaChart data={data} margin={{ top: 8, right: 8, left: -18, bottom: 0 }}>
                    <defs>
                        <linearGradient id="evrate" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stopColor={chart.accent} stopOpacity={0.45} />
                            <stop offset="100%" stopColor={chart.accent} stopOpacity={0} />
                        </linearGradient>
                    </defs>
                    <CartesianGrid stroke={chart.grid} vertical={false} />
                    <XAxis
                        dataKey="t"
                        tick={axisTick}
                        stroke={chart.grid}
                        tickFormatter={(value: number) => `-${29 - value}m`}
                        interval={5}
                    />
                    <YAxis tick={axisTick} stroke={chart.grid} allowDecimals={false} width={38} />
                    <Tooltip
                        {...tooltipLineCursor}
                        labelFormatter={(label) => {
                            const n = Number(label);
                            return Number.isFinite(n) ? `${29 - n} min ago` : "";
                        }}
                    />
                    <Area
                        type="monotone"
                        dataKey="count"
                        name="events"
                        stroke={chart.accent}
                        strokeWidth={2}
                        fill="url(#evrate)"
                    />
                    <Line
                        type="monotone"
                        dataKey="crit"
                        name="critical"
                        stroke={chart.critical}
                        strokeWidth={2}
                        dot={false}
                    />
                </AreaChart>
            </ResponsiveContainer>
        </Panel>
    );
}
