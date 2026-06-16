/* Sessions — there's no list-sessions endpoint, so this exposes the
 * highest-value control: revoke every session for this account at once. */

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import { Panel } from "@/components/Panel";
import { Button } from "@/components/Button";

export function SessionsSection() {
    const { logoutEverywhere } = useAuth();
    const navigate = useNavigate();
    const [busy, setBusy] = useState(false);

    async function signOutAll() {
        setBusy(true);
        await logoutEverywhere();
        navigate("/login", { replace: true });
    }

    return (
        <Panel title="sessions">
            <div className="space-y-2 p-1 text-[11px] text-fg-dim">
                <p>
                    If you suspect a device is compromised, revoke every active session for this
                    account. You'll need to sign in again everywhere.
                </p>
                <Button type="button" variant="danger" onClick={signOutAll} disabled={busy}>
                    {busy ? "revoking…" : "sign out everywhere"}
                </Button>
            </div>
        </Panel>
    );
}
