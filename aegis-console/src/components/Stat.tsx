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
        <div className="flex min-w-0 flex-col gap-1 border border-line bg-panel px-3.5 py-3">
            <div className="text-[10px] uppercase tracking-[1.5px] text-fg-dim">{label}</div>
            <div className={`text-[26px] font-bold leading-none ${accent ?? "text-fg"}`}>{value}</div>
            {sub && <div className="text-[10px] text-fg-dim">{sub}</div>}
        </div>
    );
}
