/** Zone divider — a 2px rule with the zone name on the left and what the zone
 *  contains on the right. It's what turns a wall of panels into three bands. */
export function SectionLabel({ children, hint }: { children: string; hint?: string }) {
    return (
        <div className="mt-6 flex items-baseline justify-between border-t-2 border-line pt-2">
            <span className="font-heading text-[13px] font-extrabold uppercase tracking-[0.18em]">
                {children}
            </span>
            {hint && (
                <span className="text-[10px] uppercase tracking-[0.14em] text-fg-dim">{hint}</span>
            )}
        </div>
    );
}
