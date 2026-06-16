/* LoginPage — email + password. On mfa_required, hand the short-lived
 * mfa_token to /mfa-challenge. Errors stay generic (no oracle for attackers). */

import { useState, type FormEvent } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import { ApiClientError } from "@/lib/api/client";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

export default function LoginPage() {
    const navigate = useNavigate();
    const { login } = useAuth();

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    async function onSubmit(e: FormEvent) {
        e.preventDefault();
        setError(null);
        setSubmitting(true);
        try {
            const res = await login(email.trim(), password);
            if (res.mfa_required && res.mfa_token) {
                navigate("/mfa-challenge", { state: { mfaToken: res.mfa_token } });
                return;
            }
            navigate("/", { replace: true });
        } catch (err) {
            const code = err instanceof ApiClientError ? err.code : "UNKNOWN";
            setError(
                code === "NETWORK_ERROR"
                    ? "Can't reach the server. Is the backend running on :3000?"
                    : "Invalid email or password.",
            );
        } finally {
            setSubmitting(false);
        }
    }

    return (
        <AuthShell barLabel="authenticate" subtitle="sign in">
            <form onSubmit={onSubmit} className="space-y-3">
                <Field label="email" type="email" autoComplete="email" required value={email}
                    onChange={(e) => setEmail(e.target.value)} />
                <Field label="password" type="password" autoComplete="current-password" required
                    value={password} onChange={(e) => setPassword(e.target.value)} />
                {error && <Notice kind="error">{error}</Notice>}
                <Button type="submit" disabled={submitting} className="w-full">
                    {submitting ? "authenticating…" : "sign in"}
                </Button>
                <div className="flex items-center justify-between pt-1 text-[10px] text-fg-dim">
                    <Link to="/forgot-password" className="hover:text-accent">forgot password?</Link>
                    <Link to="/register" className="text-accent hover:underline">register</Link>
                </div>
            </form>
        </AuthShell>
    );
}
