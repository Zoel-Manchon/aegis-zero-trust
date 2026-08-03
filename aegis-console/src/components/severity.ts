import type { Severity } from "@/types";

export const SEVERITIES: Severity[] = ["info", "low", "medium", "high", "critical"];

export const sevText: Record<Severity, string> = {
    info: "text-sev-info",
    low: "text-sev-low",
    medium: "text-sev-medium",
    high: "text-sev-high",
    critical: "text-sev-critical",
};

export const sevBg: Record<Severity, string> = {
    info: "bg-sev-info",
    low: "bg-sev-low",
    medium: "bg-sev-medium",
    high: "bg-sev-high",
    critical: "bg-sev-critical",
};

/* Chip pairs for severity badges — neutral steps for the quiet end of the ramp,
 * vermilion only from `high` up, so a red chip always means "act". */
export const sevChip: Record<Severity, string> = {
    info: "bg-neutral-200 text-neutral-800",
    low: "bg-neutral-200 text-neutral-900",
    medium: "bg-neutral-300 text-neutral-900",
    high: "bg-accent-200 text-accent-800",
    critical: "bg-accent text-bg",
};

/* Mirrors the --color-sev-* tokens in index.css. SVG fills need literal values,
 * so any change here must be made there too. */
export const sevHex: Record<Severity, string> = {
    info: "#9b9797",
    low: "#7d7979",
    medium: "#444141",
    high: "#dd2b0f",
    critical: "#ec3013",
};
