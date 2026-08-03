/* LoginPage — email + password, or a passkey (WebAuthn). On mfa_required, hand
 * the short-lived mfa_token to /mfa-challenge. Errors stay generic (no oracle). */

import { useState, type FormEvent } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/lib/auth/AuthContext";
import { ApiClientError } from "@/lib/api/client";
import { passkeysSupported } from "@/lib/auth/webauthn";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
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
        <AuthShell barLabel="Authenticate" subtitle="Sign in">
            <form onSubmit={onSubmit}>
                <div className="mb-3.5">
                    <Field
                        label="Email"
                        type="email"
                        autoComplete="email"
                        required
                        placeholder="operator@aegis.io"
                        value={email}
                        onChange={(e) => setEmail(e.target.value)}
                    />
                </div>
                <div className="mb-4.5">
                    <Field
                        label="Password"
                        type="password"
                        autoComplete="current-password"
                        required
                        placeholder="••••••••••"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                    />
                </div>

                {error && (
                    <div className="mb-3.5">
                        <Notice kind="error">{error}</Notice>
                    </div>
                )}

                <button type="submit" disabled={submitting} className="btn btn-primary btn-block">
                    {submitting ? "Authenticating…" : "Sign in →"}
                </button>
                <button
                    type="button"
                    onClick={onPasskey}
                    disabled={passkeyBusy}
                    className="btn btn-secondary btn-block"
                >
                    {passkeyBusy ? "Waiting for device…" : "Sign in with a passkey"}
                </button>

                <div className="mt-4.5 flex justify-between text-[11px] uppercase tracking-[0.1em]">
                    <Link to="/forgot-password" className="text-fg-dim hover:text-accent-700">
                        Forgot password?
                    </Link>
                    <Link to="/register" className="text-accent-700 hover:underline">
                        Register
                    </Link>
                </div>
            </form>
        </AuthShell>
    );
}
