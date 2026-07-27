/* GeoMapPanel — plots GeoIP origins on an equirectangular grid and draws arcs
 * for impossible-travel hops (matched to plotted cities by name). Pure SVG, no
 * basemap dependency, so it renders identically on any machine.
 *
 * This is the panel the demo lives or dies on, so the impossible-travel arc is
 * the one element allowed to shout: everything else — graticule, labels, idle
 * origins — is deliberately quiet so the hop reads instantly. */

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

    const meridians = [-120, -60, 0, 60, 120];
    const parallels = [-60, -30, 0, 30, 60];
    const maxCount = Math.max(1, ...points.map((p) => p.count));

    return (
        <Panel
            title="geo intelligence · live map"
            right={
                <span className="flex items-center gap-3">
                    <span className="flex items-center gap-1.5">
                        <span className="inline-block h-2 w-2 rounded-full bg-accent" />
                        {points.length} origins
                    </span>
                    <span
                        className={`flex items-center gap-1.5 ${hits.length ? "text-sev-critical" : ""}`}
                    >
                        <span className="inline-block h-px w-3 bg-current" />
                        {hits.length} impossible travel
                    </span>
                </span>
            }
        >
            <svg
                viewBox={`0 0 ${W} ${H}`}
                className="h-auto w-full"
                preserveAspectRatio="xMidYMid meet"
                role="img"
                aria-label={`World map: ${points.length} attack origins, ${hits.length} impossible-travel hops`}
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
                        <path d="M 0 0 L 10 5 L 0 10 z" fill={chart.critical} />
                    </marker>
                    <radialGradient id="origin-halo">
                        <stop offset="0%" stopColor={chart.accent} stopOpacity={0.35} />
                        <stop offset="100%" stopColor={chart.accent} stopOpacity={0} />
                    </radialGradient>
                </defs>

                <rect x={0} y={0} width={W} height={H} fill="var(--color-bg)" />

                {meridians.map((lon) => {
                    const [x] = project(0, lon);
                    return <line key={`m${lon}`} x1={x} y1={0} x2={x} y2={H} stroke={chart.grid} strokeWidth={1} />;
                })}
                {parallels.map((lat) => {
                    const [, y] = project(lat, 0);
                    return <line key={`p${lat}`} x1={0} y1={y} x2={W} y2={y} stroke={chart.grid} strokeWidth={1} />;
                })}
                {/* equator, called out because the graticule otherwise has no anchor */}
                <line x1={0} y1={H / 2} x2={W} y2={H / 2} stroke={chart.equator} strokeWidth={1} strokeDasharray="5 5" />

                {arcs.map(({ a, b }, i) => {
                    const [x1, y1] = project(a.latitude, a.longitude);
                    const [x2, y2] = project(b.latitude, b.longitude);
                    const mx = (x1 + x2) / 2;
                    const my = (y1 + y2) / 2 - Math.abs(x2 - x1) * 0.2;
                    const d = `M ${x1} ${y1} Q ${mx} ${my} ${x2} ${y2}`;
                    return (
                        <g key={`a${i}`}>
                            {/* soft underlay so the arc survives video compression */}
                            <path d={d} fill="none" stroke={chart.critical} strokeWidth={5} opacity={0.16} />
                            <path
                                className="arc-flow"
                                d={d}
                                fill="none"
                                stroke={chart.critical}
                                strokeWidth={1.75}
                                strokeDasharray="6 4"
                                markerEnd="url(#arc-head)"
                            />
                        </g>
                    );
                })}

                {points.map((p) => {
                    const [x, y] = project(p.latitude, p.longitude);
                    const r = 4 + (p.count / maxCount) * 4;
                    const flip = x > W - 120;
                    return (
                        <g key={`${p.city}-${p.country}`}>
                            <circle cx={x} cy={y} r={22} fill="url(#origin-halo)" />
                            <circle cx={x} cy={y} r={r} fill={chart.accent} />
                            <circle cx={x} cy={y} r={r + 4} fill="none" stroke={chart.accent} strokeWidth={1} opacity={0.4} />
                            <text
                                x={flip ? x - r - 6 : x + r + 6}
                                y={y + 4}
                                textAnchor={flip ? "end" : "start"}
                                fontSize={11}
                                fill={chart.fg}
                                stroke="var(--color-bg)"
                                strokeWidth={3}
                                paintOrder="stroke"
                            >
                                {p.city} · {p.count}
                            </text>
                        </g>
                    );
                })}

                {points.length === 0 && (
                    <text x={W / 2} y={H / 2} fontSize={12} textAnchor="middle" fill={chart.fgDim}>
                        No geo data yet. Launch an attack from an origin to plot it.
                    </text>
                )}
            </svg>
        </Panel>
    );
}
