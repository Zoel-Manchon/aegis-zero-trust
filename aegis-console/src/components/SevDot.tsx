import type { Severity } from "@/types";
import { sevBg, sevText } from "@/components/severity";

/* The glow uses `currentColor`, so the element also needs the matching *text*
 * colour — with only `bg-sev-*` the halo inherited the parent's text colour and
 * came out the wrong hue. */
export function SevDot({ sev }: { sev: Severity }) {
    return (
        <span
            className={`inline-block h-2 w-2 rounded-full ${sevBg[sev]} ${sevText[sev]}`}
            style={{ boxShadow: "0 0 7px currentColor" }}
        />
    );
}
