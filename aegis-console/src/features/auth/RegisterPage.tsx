/* RegisterPage — provision an account then auto sign-in. Client min-length
 * matches the backend policy (12). /register is anti-enumeration: same body
 * whether or not the email exists, so we just attempt login afterward. */

import { useState, type FormEvent } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import { authApi } from "@/lib/api/auth";
import { ApiClientError } from "@/lib/api/client";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

const MIN_PASSWORD = 12;

export default function RegisterPage() {
    const navigate = useNavigate();
    const { login } = useAuth();

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    async function onSubmit(e: FormEvent) {
        e.preventDefault();
        setError(null);
        if (password.length < MIN_PASSWORD) {
            setError(`Password must be at least ${MIN_PASSWORD} characters.`);
            return;
        }
        setSubmitting(true);
        try {
            await authApi.register({ email: email.trim(), password });
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
                    : code === "AUTH_WEAK_PASSWORD"
                      ? "That password doesn't meet the strength policy. Try a longer passphrase."
                      : "Couldn't create the account. Try a different email.",
            );
        } finally {
            setSubmitting(false);
        }
    }

    return (
        <AuthShell barLabel="Provision account" subtitle="Create account">
            <form onSubmit={onSubmit} className="space-y-3">
                <Field label="Email" type="email" autoComplete="email" required value={email}
                    onChange={(e) => setEmail(e.target.value)} />
                <Field label="Password" type="password" autoComplete="new-password" required
                    value={password} onChange={(e) => setPassword(e.target.value)}
                    hint={`min ${MIN_PASSWORD} chars · server enforces full policy`} />
                {error && <Notice kind="error">{error}</Notice>}
                <Button type="submit" disabled={submitting} className="w-full">
                    {submitting ? "Provisioning…" : "Create account"}
                </Button>
                <div className="pt-1 text-center text-[11px] text-fg-dim">
                    Already have an account?{" "}
                    <Link to="/login" className="text-accent-700 hover:underline">Sign in</Link>
                </div>
            </form>
        </AuthShell>
    );
}
