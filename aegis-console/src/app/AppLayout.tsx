/* =============================================================================
 * AppLayout — chrome for authenticated screens: brand, identity readout, nav,
 * and a live UTC clock. The identity strip surfaces the /me role + risk score
 * so the operator always knows who they are and what posture they're at.
 * ========================================================================== */

import { useEffect, useState, type ReactNode } from "react";
import { NavLink, useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";

function navClass({ isActive }: { isActive: boolean }) {
    return `border px-2.5 py-1 text-[10px] uppercase tracking-wide hover:brightness-125 ${
        isActive ? "border-line bg-panel-hi text-accent" : "border-transparent text-fg-dim"
    }`;
}

export function AppLayout({ children }: { children: ReactNode }) {
    const { user, logout } = useAuth();
    const navigate = useNavigate();
    const [clock, setClock] = useState(new Date());

    useEffect(() => {
        const t = window.setInterval(() => setClock(new Date()), 1000);
        return () => window.clearInterval(t);
    }, []);

    async function onLogout() {
        await logout();
        navigate("/login", { replace: true });
    }

    return (
        <div className="min-h-screen">
            <header className="flex items-center justify-between border-b border-line px-3.5 py-2.5">
                <div className="flex items-center gap-3.5">
                    <span className="text-[15px] font-bold tracking-[2px] text-accent">
                        aegis<span className="text-fg-dim">::</span>SOC
                    </span>
                    <nav className="flex gap-1">
                        <NavLink to="/dashboard" className={navClass}>dashboard</NavLink>
                        <NavLink to="/account" className={navClass}>account</NavLink>
                    </nav>
                </div>
                <div className="flex items-center gap-4 text-[11px] text-fg-dim">
                    <span>{clock.toISOString().replace("T", " ").slice(0, 19)}Z</span>
                    {user && (
                        <span className="flex items-center gap-2">
                            <span className="text-fg">{user.email}</span>
                            <span className="border border-line px-1.5 py-0.5 text-[9px] uppercase tracking-wide text-accent">
                                {user.role}
                            </span>
                            <span title="session risk score">risk {user.risk_score}</span>
                        </span>
                    )}
                    <button
                        onClick={onLogout}
                        className="border border-line px-2.5 py-1 text-[10px] uppercase tracking-wide text-fg-dim hover:text-sev-high hover:border-sev-high/50"
                    >
                        sign out
                    </button>
                </div>
            </header>
            <main>{children}</main>
        </div>
    );
}
