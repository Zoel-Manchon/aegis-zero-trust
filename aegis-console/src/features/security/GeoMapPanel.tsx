/* GeoMapPanel — plots GeoIP origins on an equirectangular grid and draws red
 * arcs for impossible-travel hops (matched to plotted cities by name). Pure SVG,
 * no basemap dependency. */

import { Panel } from "@/components/Panel";
import type { GeoPoint, ImpossibleTravelHit } from "@/features/security/derive";

const W = 720;
const H = 320;

function project(lat: number, lon: number): [number, number] {
    const x = ((lon + 180) / 360) * W;
    const y = ((90 - lat) / 180) * H;
    return [x, y];
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

    const graticule: number[] = [-120, -60, 0, 60, 120];
    const parallels: number[] = [-60, -30, 0, 30, 60];

    return (
        <Panel title="geo intelligence · live map" right={`${points.length} origins · ${hits.length} travel`}>
            <div className="p-1">
                <svg viewBox={`0 0 ${W} ${H}`} className="h-auto w-full" preserveAspectRatio="xMidYMid meet">
                    <rect x={0} y={0} width={W} height={H} fill="var(--color-bg, #0a0e0a)" />
                    {graticule.map((lon) => {
                        const [x] = project(0, lon);
                        return <line key={`m${lon}`} x1={x} y1={0} x2={x} y2={H} stroke="var(--color-grid, #1c241c)" strokeWidth={1} />;
                    })}
                    {parallels.map((lat) => {
                        const [, y] = project(lat, 0);
                        return <line key={`p${lat}`} x1={0} y1={y} x2={W} y2={y} stroke="var(--color-grid, #1c241c)" strokeWidth={1} />;
                    })}
                    <line x1={0} y1={H / 2} x2={W} y2={H / 2} stroke="var(--color-grid, #2a352a)" strokeWidth={1} strokeDasharray="4 4" />

                    {arcs.map(({ a, b }, i) => {
                        const [x1, y1] = project(a.latitude, a.longitude);
                        const [x2, y2] = project(b.latitude, b.longitude);
                        const mx = (x1 + x2) / 2;
                        const my = (y1 + y2) / 2 - Math.abs(x2 - x1) * 0.18;
                        return (
                            <path key={`a${i}`} d={`M ${x1} ${y1} Q ${mx} ${my} ${x2} ${y2}`}
                                fill="none" stroke="var(--color-sev-critical, #ff4040)" strokeWidth={1.5} strokeDasharray="5 3" opacity={0.85} />
                        );
                    })}

                    {points.map((p) => {
                        const [x, y] = project(p.latitude, p.longitude);
                        return (
                            <g key={`${p.city}-${p.country}`}>
                                <circle cx={x} cy={y} r={5} fill="var(--color-accent, #46e07a)" opacity={0.85} />
                                <circle cx={x} cy={y} r={9} fill="none" stroke="var(--color-accent, #46e07a)" strokeWidth={1} opacity={0.35} />
                                <text x={x + 8} y={y + 3} fontSize={9} fill="var(--color-fg-dim, #7a8a7a)">
                                    {p.city} · {p.count}
                                </text>
                            </g>
                        );
                    })}

                    {points.length === 0 && (
                        <text x={W / 2} y={H / 2} fontSize={12} textAnchor="middle" fill="var(--color-fg-dim, #7a8a7a)">
                            no geo data yet — launch an attack from an origin
                        </text>
                    )}
                </svg>
            </div>
        </Panel>
    );
}
