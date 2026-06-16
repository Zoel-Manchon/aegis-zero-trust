import type { Severity } from "@/types";
import { sevBg } from "@/components/severity";

export function SevDot({ sev }: { sev: Severity }) {
    return (
        <span
            className={`inline-block h-2 w-2 rounded-full ${sevBg[sev]}`}
            style={{ boxShadow: "0 0 8px currentColor" }}
        />
    );
}
