/* =============================================================================
 * Route table.
 *
 * Public:    /login /register /mfa-challenge /forgot-password /reset-password
 *            /verify-email
 * Protected: /dashboard (admin-gated SIEM) · /range (admin-gated red team)
 *            /account (self-service)
 * Catch-all: / and unknown → redirect by auth state.
 * ========================================================================== */

import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { ProtectedRoute } from "@/app/ProtectedRoute";
import { RoleRoute } from "@/app/RoleRoute";
import { AppLayout } from "@/app/AppLayout";
import { useAuth } from "@/lib/auth/AuthContext";

import LoginPage from "@/features/auth/LoginPage";
import RegisterPage from "@/features/auth/RegisterPage";
import MfaChallengePage from "@/features/auth/MfaChallengePage";
import ForgotPasswordPage from "@/features/auth/ForgotPasswordPage";
import ResetPasswordPage from "@/features/auth/ResetPasswordPage";
import VerifyEmailPage from "@/features/auth/VerifyEmailPage";
import AccountPage from "@/features/account/AccountPage";
import Dashboard from "@/features/security/Dashboard";
import AttackRangePage from "@/features/security/AttackRangePage";

function Booting() {
    return (
        <div className="flex min-h-screen items-center justify-center text-[11px] uppercase tracking-[0.16em] text-fg-dim">
            restoring session…
        </div>
    );
}

function RootRedirect() {
    const { accessToken, booting, role, roleResolved } = useAuth();
    if (booting) return <Booting />;
    if (!accessToken) return <Navigate to="/login" replace />;
    // Wait for /me so we send admins to the SIEM and everyone else to /account
    // — never dump a non-admin onto the admin dashboard (it 401s → refresh storm).
    if (!roleResolved) return <Booting />;
    return <Navigate to={role === "admin" ? "/dashboard" : "/account"} replace />;
}

export function AppRouter() {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<RootRedirect />} />

                <Route path="/login" element={<LoginPage />} />
                <Route path="/register" element={<RegisterPage />} />
                <Route path="/mfa-challenge" element={<MfaChallengePage />} />
                <Route path="/forgot-password" element={<ForgotPasswordPage />} />
                <Route path="/reset-password" element={<ResetPasswordPage />} />
                <Route path="/verify-email" element={<VerifyEmailPage />} />

                <Route
                    path="/dashboard"
                    element={
                        <ProtectedRoute>
                            <AppLayout>
                                <RoleRoute require="admin">
                                    <Dashboard />
                                </RoleRoute>
                            </AppLayout>
                        </ProtectedRoute>
                    }
                />
                <Route
                    path="/range"
                    element={
                        <ProtectedRoute>
                            <AppLayout>
                                <RoleRoute require="admin">
                                    <AttackRangePage />
                                </RoleRoute>
                            </AppLayout>
                        </ProtectedRoute>
                    }
                />
                <Route
                    path="/account"
                    element={
                        <ProtectedRoute>
                            <AppLayout>
                                <AccountPage />
                            </AppLayout>
                        </ProtectedRoute>
                    }
                />

                <Route path="*" element={<RootRedirect />} />
            </Routes>
        </BrowserRouter>
    );
}
