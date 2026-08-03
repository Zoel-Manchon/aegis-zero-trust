import type { Severity } from "@/types";
import { sevBg } from "@/components/severity";

/** Severity marker. Square, not a dot: nothing in this system is rounded, and
 *  the square reads at 8px in a dense feed where a circle turns to mush. */
export function SevDot({ sev }: { sev: Severity }) {
    return <span className={`inline-block h-2 w-2 ${sevBg[sev]}`} />;
}
