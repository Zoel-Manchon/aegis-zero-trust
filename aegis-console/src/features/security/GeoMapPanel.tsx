/* GeoMapPanel — plots GeoIP origins on an equirectangular graticule and draws
 * arcs for impossible-travel hops (matched to plotted cities by name). Pure SVG,
 * no basemap dependency, so it renders identically on any machine.
 *
 * There is no coastline on purpose: this is a plot, not a map, and the graticule
 * plus the labelled markers carry all the information an operator needs. The
 * impossible-travel arc is the one element allowed to shout — everything else,
 * grid included, is deliberately quiet so the hop reads instantly. */

import { Panel } from "@/components/Panel";
import { chart } from "@/features/security/chartTheme";
import type { GeoPoint, ImpossibleTravelHit } from "@/features/security/derive";

const W = 720;
const H = 320;

function project(lat: number, lon: number): [number, number] {
    return [((lon + 180) / 360) * W, ((90 - lat) / 180) * H];
}

export function GeoMapPanel({ points, hits }: { points: GeoPoint[]; hits: ImpossibleTravelHit[] }) {
    const byCity = new Map(points.map((p) => [p.city.toLowerCase(), p]));
    const arcs = hits
        .map((h) => {
            const a = byCity.get(h.from.split(",")[0].trim().toLowerCase());
            const b = byCity.get(h.to.split(",")[0].trim().toLowerCase());
            return a && b ? { a, b, h } : null;
        })
        .filter((x): x is { a: GeoPoint; b: GeoPoint; h: ImpossibleTravelHit } => x !== null);

    const meridians = [-150, -120, -90, -60, -30, 30, 60, 90, 120, 150];
    const parallels = [-60, -30, 30, 60];
    const maxCount = Math.max(1, ...points.map((p) => p.count));

    return (
        <Panel
            title="Origin plot"
            right={`equirectangular · ${points.length} origins`}
            bodyClassName="p-3"
        >
            <svg
                viewBox={`0 0 ${W} ${H}`}
                className="h-auto w-full border border-line"
                preserveAspectRatio="xMidYMid meet"
                role="img"
                aria-label={`World plot: ${points.length} attack origins, ${arcs.length} impossible-travel hops`}
            >
                <defs>
                    <marker
                        id="arc-head"
                        viewBox="0 0 10 10"
                        refX="8"
                        refY="5"
                        markerWidth="5"
                        markerHeight="5"
                        orient="auto-start-reverse"
                    >
                        <path d="M 0 0 L 10 5 L 0 10 z" fill={chart.accent} />
                    </marker>
                </defs>

                <rect x={0} y={0} width={W} height={H} fill={chart.paper} />

                {meridians.map((lon) => {
                    const [x] = project(0, lon);
                    return <line key={`m${lon}`} x1={x} y1={0} x2={x} y2={H} stroke={chart.grid} strokeWidth={1} />;
                })}
                {parallels.map((lat) => {
                    const [, y] = project(lat, 0);
                    return <line key={`p${lat}`} x1={0} y1={y} x2={W} y2={y} stroke={chart.grid} strokeWidth={1} />;
                })}
                {/* prime meridian + equator: the graticule needs one anchored cross */}
                <line x1={W / 2} y1={0} x2={W / 2} y2={H} stroke={chart.equator} strokeWidth={1} />
                <line x1={0} y1={H / 2} x2={W} y2={H / 2} stroke={chart.equator} strokeWidth={1} />

                {arcs.map(({ a, b }, i) => {
                    const [x1, y1] = project(a.latitude, a.longitude);
                    const [x2, y2] = project(b.latitude, b.longitude);
                    const mx = (x1 + x2) / 2;
                    const my = (y1 + y2) / 2 - Math.abs(x2 - x1) * 0.2;
                    const d = `M ${x1} ${y1} Q ${mx} ${my} ${x2} ${y2}`;
                    return (
                        <g key={`a${i}`}>
                            <path d={d} fill="none" stroke={chart.accent} strokeWidth={5} opacity={0.16} />
                            <path
                                className="arc-flow"
                                d={d}
                                fill="none"
                                stroke={chart.accent}
                                strokeWidth={1.75}
                                strokeDasharray="6 4"
                                markerEnd="url(#arc-head)"
                            />
                        </g>
                    );
                })}

                {points.map((p) => {
                    const [x, y] = project(p.latitude, p.longitude);
                    const s = 7 + Math.round((p.count / maxCount) * 12);
                    const hot = p.count / maxCount > 0.6;
                    const flip = x > W - 130;
                    return (
                        <g key={`${p.city}-${p.country}`}>
                            {/* square markers, sized by volume — no circles anywhere in this system */}
                            <rect
                                x={x - s / 2}
                                y={y - s / 2}
                                width={s}
                                height={s}
                                fill={hot ? chart.accent : chart.marker}
                                stroke={chart.paper}
                                strokeWidth={1}
                            />
                            <text
                                x={flip ? x - s / 2 - 6 : x + s / 2 + 6}
                                y={y + 3}
                                textAnchor={flip ? "end" : "start"}
                                fontSize={9}
                                letterSpacing="0.1em"
                                fill={chart.fgDim}
                                stroke={chart.paper}
                                strokeWidth={3}
                                paintOrder="stroke"
                            >
                                {p.city.toUpperCase()} {p.count}
                            </text>
                        </g>
                    );
                })}

                {points.length === 0 && (
                    <text x={W / 2} y={H / 2} fontSize={11} textAnchor="middle" fill={chart.fgDim}>
                        No geo data yet. Launch a scenario from an origin to plot it.
                    </text>
                )}
            </svg>
        </Panel>
    );
}
