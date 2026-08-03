/* Sessions — the API exposes no list-sessions endpoint, so this panel shows the
 * session you're in (from /me) and the one control that matters: revoke every
 * session for the account at once. No invented device inventory. */

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import { Panel } from "@/components/Panel";
import { Button } from "@/components/Button";

function riskClass(risk: number): string {
    return risk > 60 ? "text-accent" : risk > 30 ? "text-accent-600" : "text-fg-dim";
}

export function SessionsSection() {
    const { user, logoutEverywhere } = useAuth();
    const navigate = useNavigate();
    const [busy, setBusy] = useState(false);

    async function signOutAll() {
        setBusy(true);
        await logoutEverywhere();
        navigate("/login", { replace: true });
    }

    const risk = user?.risk_score ?? 0;

    return (
        <Panel title="Active sessions" right="this device" bodyClassName="">
            <table className="table text-[12px]">
                <thead>
                    <tr>
                        <th className="px-3 py-1.5 text-left text-[10px] uppercase tracking-[0.14em]">
                            Account
                        </th>
                        <th className="px-2 py-1.5 text-left text-[10px] uppercase tracking-[0.14em]">
                            Role
                        </th>
                        <th className="px-2 py-1.5 text-left text-[10px] uppercase tracking-[0.14em]">
                            Risk
                        </th>
                        <th className="px-3 py-1.5 text-right text-[10px] uppercase tracking-[0.14em]">
                            Action
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr className={risk > 60 ? "bg-accent-100" : ""}>
                        <td className="px-3 py-2 font-heading font-extrabold">
                            {user?.email ?? "—"}
                        </td>
                        <td className="px-2 py-2 text-fg-dim">{user?.role ?? "—"}</td>
                        <td className={`px-2 py-2 ${riskClass(risk)}`}>{risk}</td>
                        <td className="px-3 py-2 text-right text-fg-dim">This device</td>
                    </tr>
                </tbody>
            </table>
            <div className="flex flex-wrap items-center gap-3 border-t border-line p-3.5">
                <Button type="button" onClick={signOutAll} disabled={busy}>
                    {busy ? "Revoking…" : "Sign out everywhere"}
                </Button>
                <p className="max-w-[52ch] text-[11px] leading-[1.5] text-fg-dim">
                    Revokes every refresh-token family for this account. Any other device is signed
                    out on its next request.
                </p>
            </div>
        </Panel>
    );
}
