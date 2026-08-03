import type { ReactNode } from "react";

/**
 * Panel — the single container the whole console is built from: a hairline box
 * on paper with a micro-label header. `rule` thickens the header underline to
 * 2px, which marks a panel as the primary one in its zone.
 */
export function Panel({
    title,
    right,
    children,
    className = "",
    bodyClassName = "p-3",
    rule = false,
}: {
    title: ReactNode;
    right?: ReactNode;
    children: ReactNode;
    className?: string;
    bodyClassName?: string;
    rule?: boolean;
}) {
    return (
        <div className={`border border-line bg-panel ${className}`}>
            <div
                className={`flex items-center justify-between gap-3 px-3 py-2 ${
                    rule ? "border-b-2 border-line" : "border-b border-line"
                }`}
            >
                <span className="micro">{title}</span>
                {right !== undefined && <span className="micro text-fg-dim">{right}</span>}
            </div>
            <div className={bodyClassName}>{children}</div>
        </div>
    );
}
