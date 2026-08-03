import type { ReactNode } from "react";

/** Inline message. The accent rail on the left is the only ornament — it marks
 *  the line as system output rather than page copy. */
export function Notice({ kind, children }: { kind: "error" | "ok" | "info"; children: ReactNode }) {
    const cls =
        kind === "error"
            ? "border-accent bg-accent-100 text-accent-800"
            : kind === "ok"
              ? "border-neutral-800 bg-neutral-200 text-neutral-900"
              : "border-neutral-500 bg-neutral-200 text-neutral-800";
    return <div className={`border-l-[3px] px-2.5 py-2 text-[12px] ${cls}`}>{children}</div>;
}
