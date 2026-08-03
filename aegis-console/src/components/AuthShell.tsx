import type { ReactNode } from "react";

/* =============================================================================
 * AuthShell — the split screen every unauthenticated view sits inside.
 *
 * Left: a full-bleed vermilion field carrying the product's one sentence —
 * "never trust, always verify" — set in 66px Archivo 800. It is the only place
 * in the whole console where the accent is used as a surface rather than a
 * signal, which is what makes the sign-in door feel like a door.
 * Right: the form, on paper, at 380px.
 * ========================================================================== */

const CAPABILITIES = ["Mini-SIEM", "WebAuthn", "GeoIP", "Impossible travel"];

export function AuthShell({
    barLabel,
    subtitle,
    children,
    footer,
}: {
    /** the eyebrow above the form heading, e.g. "authenticate" */
    barLabel: string;
    /** the form heading itself, e.g. "sign in" */
    subtitle: string;
    children: ReactNode;
    footer?: ReactNode;
}) {
    return (
        <div className="grid min-h-screen grid-cols-1 lg:grid-cols-2">
            <aside className="hidden flex-col justify-between bg-accent p-12 text-bg lg:flex">
                <div className="font-heading text-[15px] font-extrabold tracking-[0.22em]">
                    AEGIS<span className="opacity-60">::</span>SOC
                </div>
                <div>
                    <h1 className="font-heading text-[66px] font-extrabold uppercase leading-[0.92] tracking-[-0.03em]">
                        Never
                        <br />
                        trust.
                        <br />
                        Always
                        <br />
                        verify.
                    </h1>
                    <p className="mt-6 max-w-[30ch] text-[14px] leading-[1.5] opacity-90">
                        Zero-trust operations console. Continuous authentication, session risk
                        scoring and live attack telemetry.
                    </p>
                </div>
                <div className="flex flex-wrap gap-8 text-[11px] uppercase tracking-[0.14em] opacity-85">
                    {CAPABILITIES.map((c) => (
                        <span key={c}>{c}</span>
                    ))}
                </div>
            </aside>

            <main className="flex items-center px-6 py-12 lg:px-12">
                <div className="w-full max-w-[380px]">
                    <div className="border-b-2 border-line pb-2.5 text-[11px] uppercase tracking-[0.18em] text-fg-dim">
                        {barLabel}
                    </div>
                    <h2 className="mb-6 mt-5 text-[34px]">{subtitle}</h2>
                    {children}
                    <div className="mt-10 border-t-2 border-line pt-2.5 text-[11px] text-fg-dim">
                        {footer ?? "Aegis zero-trust lab · every request is re-verified server-side."}
                    </div>
                </div>
            </main>
        </div>
    );
}
