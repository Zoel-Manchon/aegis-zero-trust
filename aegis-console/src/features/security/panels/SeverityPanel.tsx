import { Bar, BarChart, CartesianGrid, Cell, LabelList, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Panel } from "@/components/Panel";
import { sevHex } from "@/components/severity";
import { axisTick, chart, tooltip } from "@/features/security/chartTheme";
import type { SevSlice } from "@/features/security/derive";

export function SeverityPanel({ data }: { data: SevSlice[] }) {
    const total = data.reduce((n, d) => n + d.count, 0);
    return (
        <Panel title="severity distribution" right={`${total} events`}>
            <ResponsiveContainer width="100%" height={180}>
                <BarChart data={data} margin={{ top: 16, right: 8, left: -18, bottom: 0 }}>
                    <CartesianGrid stroke={chart.grid} vertical={false} />
                    <XAxis dataKey="sev" tick={axisTick} stroke={chart.grid} />
                    <YAxis tick={axisTick} stroke={chart.grid} allowDecimals={false} width={38} />
                    <Tooltip {...tooltip} />
                    <Bar dataKey="count" name="events" maxBarSize={54}>
                        {data.map((d) => (
                            <Cell key={d.sev} fill={sevHex[d.sev]} />
                        ))}
                        {/* The count on top of the bar: readable even when the
                          * chart is a thumbnail in a compressed video. */}
                        <LabelList
                            dataKey="count"
                            position="top"
                            offset={6}
                            fill={chart.fgDim}
                            fontSize={11}
                            formatter={(v) => {
                                const n = Number(v);
                                return Number.isFinite(n) && n > 0 ? String(n) : "";
                            }}
                        />
                    </Bar>
                </BarChart>
            </ResponsiveContainer>
        </Panel>
    );
}
