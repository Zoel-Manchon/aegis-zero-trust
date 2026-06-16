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

export const sevHex: Record<Severity, string> = {
    info: "#5b8cff",
    low: "#3dd6ff",
    medium: "#ffd23d",
    high: "#ff8a3d",
    critical: "#ff3b3b",
};
