/* =============================================================================
 * AppLayout — chrome for authenticated screens: brand, nav, identity readout,
 * and a live UTC clock. The identity strip surfaces the /me role + risk score
 * so the operator always knows who they are and what posture they're at.
 *
 * Nav is a row of hard-edged buttons; the active one is filled vermilion. That
 * single filled block is the only accent in the header, so "where am I" is
 * answerable at a glance from across the desk.
 * ========================================================================== */

import { useEffect, useState, type ReactNode } from "react";
import { NavLink, useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";

function navClass({ isActive }: { isActive: boolean }) {
    return `cursor-pointer border-0 px-3 py-1.5 font-heading text-[11px] font-extrabold uppercase tracking-[0.14em] ${
        isActive ? "bg-accent text-bg" : "bg-transparent text-fg-dim hover:text-fg"
    }`;
}

export function AppLayout({ children }: { children: ReactNode }) {
    const { user, role, logout } = useAuth();
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
            <header className="flex h-[52px] items-center justify-between gap-6 border-b-2 border-line bg-panel px-4">
                <div className="flex items-center gap-7">
                    <span className="font-heading text-[15px] font-extrabold tracking-[0.2em] text-accent">
                        AEGIS<span className="text-neutral-500">::</span>
                        <span className="text-fg">SOC</span>
                    </span>
                    <nav className="flex gap-0.5">
                        {role === "admin" && (
                            <>
                                <NavLink to="/dashboard" className={navClass}>Console</NavLink>
                                <NavLink to="/range" className={navClass}>Attack range</NavLink>
                            </>
                        )}
                        <NavLink to="/account" className={navClass}>Account</NavLink>
                    </nav>
                </div>
                <div className="flex items-center gap-4 text-[11px] text-fg-dim">
                    <span className="hidden tracking-[0.06em] md:inline">
                        {clock.toISOString().replace("T", " ").slice(0, 19)}Z
                    </span>
                    {user && (
                        <span className="flex items-center gap-2">
                            <span className="hidden text-fg sm:inline">{user.email}</span>
                            <span className="tag tag-accent text-[10px]">{user.role}</span>
                            <span title="session risk score">risk {user.risk_score}</span>
                        </span>
                    )}
                    <button
                        onClick={onLogout}
                        className="btn btn-secondary btn-micro"
                    >
                        Sign out
                    </button>
                </div>
            </header>
            <main>{children}</main>
        </div>
    );
}
