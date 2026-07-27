import { Bar, BarChart, Cell, LabelList, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Panel } from "@/components/Panel";
import { axisTick, chart, tooltip } from "@/features/security/chartTheme";
import type { EventTypeCount } from "@/features/security/derive";

/* Horizontal breakdown of which event types are firing — populates as you run
 * attacks, so the scenarios (fingerprint spoof, session hijack, …) show up here
 * distinctly. Attack-indicating types are drawn in the warn colour. */
export function EventTypePanel({ data }: { data: EventTypeCount[] }) {
    return (
        <Panel title="attack vectors — event types" right="by volume">
            {data.length === 0 ? (
                <p className="px-1 py-10 text-center text-[11px] text-fg-dim">
                    No events yet. Launch an attack from the range above to populate this chart.
                </p>
            ) : (
                <ResponsiveContainer width="100%" height={Math.max(140, data.length * 30)}>
                    <BarChart data={data} layout="vertical" margin={{ top: 4, right: 34, left: 4, bottom: 0 }}>
                        <XAxis type="number" tick={axisTick} stroke={chart.grid} allowDecimals={false} />
                        <YAxis
                            type="category"
                            dataKey="label"
                            width={160}
                            tick={{ fill: chart.fg, fontSize: 11 }}
                            stroke={chart.grid}
                        />
                        <Tooltip {...tooltip} />
                        <Bar dataKey="count" name="events" radius={[0, 2, 2, 0]} maxBarSize={18}>
                            {data.map((d) => (
                                <Cell key={d.type} fill={d.hostile ? chart.high : chart.accent} />
                            ))}
                            <LabelList
                                dataKey="count"
                                position="right"
                                offset={8}
                                fill={chart.fgDim}
                                fontSize={11}
                            />
                        </Bar>
                    </BarChart>
                </ResponsiveContainer>
            )}
        </Panel>
    );
}
