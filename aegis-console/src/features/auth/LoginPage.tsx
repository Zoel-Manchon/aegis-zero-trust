/* LoginPage — email + password, or a passkey (WebAuthn). On mfa_required, hand
 * the short-lived mfa_token to /mfa-challenge. Errors stay generic (no oracle). */

import { useState, type FormEvent } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import { ApiClientError } from "@/lib/api/client";
import { passkeysSupported } from "@/lib/auth/webauthn";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

export default function LoginPage() {
    const navigate = useNavigate();
    const { login, loginWithPasskey } = useAuth();

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [passkeyBusy, setPasskeyBusy] = useState(false);
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

    async function onPasskey() {
        setError(null);
        if (!email.trim()) {
            setError("Enter your email, then sign in with a passkey.");
            return;
        }
        if (!passkeysSupported()) {
            setError("This browser doesn't support passkeys.");
            return;
        }
        setPasskeyBusy(true);
        try {
            await loginWithPasskey(email.trim());
            navigate("/", { replace: true });
        } catch (err) {
            const name = err instanceof Error ? err.name : "";
            setError(
                name === "NotAllowedError"
                    ? "Passkey sign-in cancelled."
                    : "Couldn't sign in with a passkey.",
            );
        } finally {
            setPasskeyBusy(false);
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

                <div className="flex items-center gap-2 py-0.5 text-[10px] uppercase text-fg-dim">
                    <span className="h-px flex-1 bg-grid" />
                    or
                    <span className="h-px flex-1 bg-grid" />
                </div>
                <Button type="button" variant="ghost" onClick={onPasskey}
                    disabled={passkeyBusy} className="w-full">
                    {passkeyBusy ? "waiting for device…" : "sign in with a passkey"}
                </Button>

                <div className="flex items-center justify-between pt-1 text-[10px] text-fg-dim">
                    <Link to="/forgot-password" className="hover:text-accent">forgot password?</Link>
                    <Link to="/register" className="text-accent hover:underline">register</Link>
                </div>
            </form>
        </AuthShell>
    );
}
