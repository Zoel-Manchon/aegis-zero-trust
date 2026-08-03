/* The red-team screen. Split out of the dashboard so the console stays a pure
 * read surface and this one is unambiguously the place where you fire things. */

import { SubBar } from "@/app/SubBar";
import { AttackRange } from "@/features/security/AttackRange";

export default function AttackRangePage() {
    return (
        <>
            <SubBar label="red team · attack range" />
            <AttackRange />
        </>
    );
}
