/** Thin zone divider — groups the dense panel grid into labeled bands. */
export function SectionLabel({ children, hint }: { children: string; hint?: string }) {
    return (
        <div className="mb-2 mt-4 flex items-center gap-2">
            <span className="inline-block h-3 w-[3px] bg-accent" />
            <span className="text-[10px] font-semibold uppercase tracking-[2px] text-fg">{children}</span>
            {hint && <span className="text-[9px] uppercase tracking-wide text-fg-dim">· {hint}</span>}
            <span className="ml-1 h-px flex-1 bg-line" />
        </div>
    );
}
