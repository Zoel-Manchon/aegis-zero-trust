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

/* Mirrors the --color-sev-* tokens in index.css. SVG fills need literal values,
 * so any change here must be made there too. Contrast on --color-panel:
 * info 7.2:1 · low 11.1:1 · medium 13.5:1 · high 9.3:1 · critical 6.9:1. */
export const sevHex: Record<Severity, string> = {
    info: "#7aa2d6",
    low: "#4fd6ee",
    medium: "#ffd45e",
    high: "#ff9d52",
    critical: "#ff6b6b",
};
