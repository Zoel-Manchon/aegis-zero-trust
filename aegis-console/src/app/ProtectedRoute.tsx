/* =============================================================================
 * ProtectedRoute — re-evaluates auth on every navigation.
 *
 * If the user has no access token AND no in-progress silent refresh, redirect
 * to /login. While the boot-time silent refresh is in progress, render a
 * minimal loading state so we don't flash the login page on every reload.
 *
 * ZERO-TRUST: this guard runs on every route change. There is no "logged in
 * once, trusted forever" — every navigation re-checks state. (The Rust
 * middleware does the same on every request.)
 * ========================================================================== */

import type { ReactNode } from "react";
import { Navigate, useLocation } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";

export function ProtectedRoute({ children }: { children: ReactNode }) {
    const { accessToken, booting } = useAuth();
    const location = useLocation();

    if (booting) {
        return (
            <div className="flex min-h-screen items-center justify-center text-[11px] uppercase tracking-[0.16em] text-fg-dim">
                restoring session…
            </div>
        );
    }

    if (!accessToken) {
        // Preserve the path the user tried to reach, so we can land them back
        // there after a successful login (a nice-to-have we'll use in 6c).
        return <Navigate to="/login" replace state={{ from: location }} />;
    }

    return <>{children}</>;
}
