/* MfaChallengePage — exchange the login mfa_token + a 6-digit TOTP code for a
 * full session. The mfa_token arrives via router state (never the URL); if the
 * page is hit directly without one, bounce to /login. */

import { useState, type FormEvent } from "react";
import { Navigate, useLocation, useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import { ApiClientError } from "@/lib/api/client";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

export default function MfaChallengePage() {
    const navigate = useNavigate();
    const location = useLocation();
    const { completeMfa } = useAuth();
    const mfaToken = (location.state as { mfaToken?: string } | null)?.mfaToken;

    const [code, setCode] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    if (!mfaToken) return <Navigate to="/login" replace />;

    async function onSubmit(e: FormEvent) {
        e.preventDefault();
        setError(null);
        setSubmitting(true);
        try {
            await completeMfa(mfaToken as string, code.trim());
            navigate("/", { replace: true });
        } catch (err) {
            const code2 = err instanceof ApiClientError ? err.code : "UNKNOWN";
            setError(
                code2 === "NETWORK_ERROR"
                    ? "Can't reach the server."
                    : "That code didn't verify. Check your authenticator and try again.",
            );
        } finally {
            setSubmitting(false);
        }
    }

    return (
        <AuthShell barLabel="step-up · mfa" subtitle="enter authenticator code">
            <form onSubmit={onSubmit} className="space-y-3">
                <Field
                    label="6-digit code"
                    inputMode="numeric"
                    autoComplete="one-time-code"
                    pattern="[0-9]*"
                    maxLength={6}
                    required
                    autoFocus
                    value={code}
                    onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                />
                {error && <Notice kind="error">{error}</Notice>}
                <Button type="submit" disabled={submitting || code.length !== 6} className="w-full">
                    {submitting ? "verifying…" : "verify"}
                </Button>
            </form>
        </AuthShell>
    );
}
