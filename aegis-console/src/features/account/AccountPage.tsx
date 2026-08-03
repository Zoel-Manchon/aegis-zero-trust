/* AccountPage — self-service security center: MFA, passkeys, sessions. */

import { SubBar } from "@/app/SubBar";
import { MfaSection } from "@/features/account/MfaSection";
import { PasskeysSection } from "@/features/account/PasskeysSection";
import { SessionsSection } from "@/features/account/SessionsSection";

export default function AccountPage() {
    return (
        <>
            <SubBar label="account · security center" />
            <div className="max-w-[900px] space-y-3 p-4">
                <MfaSection />
                <PasskeysSection />
                <SessionsSection />
            </div>
        </>
    );
}
