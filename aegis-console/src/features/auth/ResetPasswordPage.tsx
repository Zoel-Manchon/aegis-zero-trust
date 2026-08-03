/* ResetPasswordPage — consume the reset token (from the ?token= link) plus a
 * new password. Token is single-use and hashed server-side. */

import { useMemo, useState, type FormEvent } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { passwordApi } from "@/lib/api/auth";
import { ApiClientError } from "@/lib/api/client";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

const MIN_PASSWORD = 12;

export default function ResetPasswordPage() {
    const [params] = useSearchParams();
    const navigate = useNavigate();
    const token = useMemo(() => params.get("token") ?? "", [params]);

    const [password, setPassword] = useState("");
    const [confirm, setConfirm] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [done, setDone] = useState(false);

    async function onSubmit(e: FormEvent) {
        e.preventDefault();
        setError(null);
        if (password.length < MIN_PASSWORD) {
            setError(`Password must be at least ${MIN_PASSWORD} characters.`);
            return;
        }
        if (password !== confirm) {
            setError("Passwords don't match.");
            return;
        }
        setSubmitting(true);
        try {
            await passwordApi.reset({ token, new_password: password });
            setDone(true);
            setTimeout(() => navigate("/login", { replace: true }), 1500);
        } catch (err) {
            const code = err instanceof ApiClientError ? err.code : "UNKNOWN";
            setError(
                code === "NETWORK_ERROR"
                    ? "Can't reach the server."
                    : "This reset link is invalid or expired. Request a new one.",
            );
        } finally {
            setSubmitting(false);
        }
    }

    if (!token) {
        return (
            <AuthShell barLabel="Recover access" subtitle="Reset password">
                <Notice kind="error">
                    Missing reset token. Open the link from your email, or{" "}
                    <Link to="/forgot-password" className="underline">request a new one</Link>.
                </Notice>
            </AuthShell>
        );
    }

    return (
        <AuthShell barLabel="Recover access" subtitle="New password">
            {done ? (
                <Notice kind="ok">Password updated. Redirecting to sign in…</Notice>
            ) : (
                <form onSubmit={onSubmit} className="space-y-3">
                    <Field label="New password" type="password" autoComplete="new-password" required
                        value={password} onChange={(e) => setPassword(e.target.value)}
                        hint={`min ${MIN_PASSWORD} chars`} />
                    <Field label="Confirm password" type="password" autoComplete="new-password" required
                        value={confirm} onChange={(e) => setConfirm(e.target.value)} />
                    {error && <Notice kind="error">{error}</Notice>}
                    <Button type="submit" disabled={submitting} className="w-full">
                        {submitting ? "Updating…" : "Update password"}
                    </Button>
                </form>
            )}
        </AuthShell>
    );
}
