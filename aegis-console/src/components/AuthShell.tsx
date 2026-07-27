import type { ReactNode } from "react";

/** The bordered terminal card every auth screen sits inside. */
export function AuthShell({
    barLabel,
    subtitle,
    children,
}: {
    barLabel: string;
    subtitle: string;
    children: ReactNode;
}) {
    return (
        <div className="flex min-h-screen items-center justify-center px-4">
            <div className="w-full max-w-sm border border-line bg-panel">
                <div className="border-b border-line bg-panel-hi px-3 py-2">
                    <span className="text-[10px] font-semibold uppercase tracking-[1.5px] text-fg">
                        {barLabel}
                    </span>
                </div>
                <div className="space-y-3 p-4">
                    <div className="mb-2 text-center">
                        <div className="text-[15px] font-bold tracking-[2px] text-accent">
                            aegis<span className="text-fg-dim">::</span>SOC
                        </div>
                        <div className="text-[10px] uppercase text-fg-dim">{subtitle}</div>
                    </div>
                    {children}
                </div>
            </div>
        </div>
    );
}
