/* =============================================================================
 * RoleRoute — front-end RBAC gate. The backend is the real authority (every
 * request re-checks the role); this avoids mounting a surface the user will
 * only ever get 401s from — which, for the admin SIEM, would otherwise trigger
 * a refresh storm (the backend returns 401, not 403, for RBAC denial).
 *
 * If the role was changed directly in Postgres while the SPA is open, the
 * cached /me result can be stale. This gate performs one just-in-time identity
 * refresh before denying access so DB role promotions are picked up without
 * requiring logout/login.
 * ========================================================================== */

import { useEffect, useRef, type ReactNode } from "react";
import { Link } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import type { UserRole } from "@/types";

function Center({ children }: { children: ReactNode }) {
    return <div className="flex min-h-[60vh] items-center justify-center px-4">{children}</div>;
}

function Card({ tone, title, body }: { tone: "high" | "dim"; title: string; body: ReactNode }) {
    const text = tone === "high" ? "text-accent" : "text-fg-dim";
    return (
        <div className="w-full max-w-md border border-line bg-panel">
            <div className={`micro border-b-2 border-line px-3 py-2 ${text}`}>{title}</div>
            <div className="space-y-2 p-4 text-[12px] leading-[1.6] text-fg-dim">{body}</div>
        </div>
    );
}

export function RoleRoute({ require: required, children }: { require: UserRole; children: ReactNode }) {
    const { role, roleResolved, refreshUser } = useAuth();
    const refreshedOnce = useRef(false);

    useEffect(() => {
        if (!roleResolved || role === required || refreshedOnce.current) return;
        refreshedOnce.current = true;
        void refreshUser();
    }, [role, roleResolved, required, refreshUser]);

    if (!roleResolved || (role !== required && !refreshedOnce.current)) {
        return <Center><span className="text-[11px] uppercase tracking-[0.16em] text-fg-dim">resolving role…</span></Center>;
    }

    if (role === required) return <>{children}</>;

    if (role === null) {
        return (
            <Center>
                <Card
                    tone="dim"
                    title="role unavailable"
                    body={
                        <>
                            <p>Couldn't confirm your role from <span className="text-fg">GET /me</span>. Check that the API is reachable and that your session is still valid.</p>
                            <p>Try a hard refresh, or sign in again after changing the database role.</p>
                        </>
                    }
                />
            </Center>
        );
    }

    return (
        <Center>
            <Card
                tone="high"
                title="access denied"
                body={
                    <>
                        <p>This console requires the <span className="text-fg">{required}</span> role; your account is <span className="text-fg">{role}</span>.</p>
                        <p>If you changed <span className="text-fg">users.user_role</span> manually, run <span className="text-fg">UPDATE users SET user_role = 'admin' WHERE email = '...'</span>, then hard-refresh or sign in again.</p>
                        <p><Link to="/account" className="text-accent-700 hover:underline">Go to account</Link></p>
                    </>
                }
            />
        </Center>
    );
}
