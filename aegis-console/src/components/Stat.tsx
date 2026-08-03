/** One cell of the KPI strip. The number is the loudest type on the page — 34px
 *  Archivo 800 — because the strip is what an operator reads from across a room. */
export function Stat({
    label,
    value,
    accent,
    sub,
}: {
    label: string;
    value: number | string;
    accent?: string;
    sub?: string;
}) {
    return (
        <div className="min-w-0 border-l border-line px-3.5 py-3 first:border-l-0">
            <div className="truncate text-[10px] uppercase tracking-[0.16em] text-fg-dim">{label}</div>
            <div
                className={`font-heading text-[34px] font-extrabold leading-[1.1] tracking-[-0.02em] ${accent ?? "text-fg"}`}
            >
                {value}
            </div>
            {sub && (
                <div className="truncate text-[10px] uppercase tracking-[0.1em] text-fg-mute">{sub}</div>
            )}
        </div>
    );
}
