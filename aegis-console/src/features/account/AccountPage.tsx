/* AccountPage — self-service security center: MFA, passkeys, sessions. */

import { MfaSection } from "@/features/account/MfaSection";
import { PasskeysSection } from "@/features/account/PasskeysSection";
import { SessionsSection } from "@/features/account/SessionsSection";

export default function AccountPage() {
    return (
        <div className="mx-auto max-w-3xl space-y-3 p-3.5">
            <div className="border-b border-line pb-2 text-[10px] uppercase tracking-[1.5px] text-fg-dim">
                account · security
            </div>
            <MfaSection />
            <PasskeysSection />
            <SessionsSection />
        </div>
    );
}
