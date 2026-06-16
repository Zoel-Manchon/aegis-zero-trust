export function Notice({ kind, children }: { kind: "error" | "ok" | "info"; children: React.ReactNode }) {
    const cls =
        kind === "error"
            ? "border-sev-critical/40 bg-sev-critical/10 text-sev-critical"
            : kind === "ok"
              ? "border-accent/40 bg-accent/10 text-accent"
              : "border-line bg-panel-hi text-fg-dim";
    return <div className={`border px-2 py-1.5 text-[11px] ${cls}`}>{children}</div>;
}
