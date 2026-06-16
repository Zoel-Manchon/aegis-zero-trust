import type { ReactNode } from "react";

export function Panel({
    title,
    right,
    children,
    className = "",
}: {
    title: string;
    right?: ReactNode;
    children: ReactNode;
    className?: string;
}) {
    return (
        <div className={`border border-line bg-panel ${className}`}>
            <div className="flex items-center justify-between border-b border-line bg-panel-hi px-3 py-2">
                <span className="text-[10px] font-semibold uppercase tracking-[1.5px]">{title}</span>
                {right && <div className="text-[10px] text-fg-dim">{right}</div>}
            </div>
            <div className="p-2.5">{children}</div>
        </div>
    );
}
