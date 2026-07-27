/* ForgotPasswordPage — request a reset link. Anti-enumeration: the UI always
 * confirms "if an account exists, a link is on its way," regardless of the
 * backend's response, so it never reveals whether an email is registered. */

import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { passwordApi } from "@/lib/api/auth";
import { ApiClientError } from "@/lib/api/client";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

export default function ForgotPasswordPage() {
    const [email, setEmail] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [sent, setSent] = useState(false);
    const [error, setError] = useState<string | null>(null);

    async function onSubmit(e: FormEvent) {
        e.preventDefault();
        setError(null);
        setSubmitting(true);
        try {
            await passwordApi.forgot({ email: email.trim() });
        } catch (err) {
            // Only surface transport failures; an existence-revealing error is
            // deliberately swallowed to preserve anti-enumeration.
            if (err instanceof ApiClientError && err.code === "NETWORK_ERROR") {
                setError("Can't reach the server.");
                setSubmitting(false);
                return;
            }
        }
        setSent(true);
        setSubmitting(false);
    }

    return (
        <AuthShell barLabel="recover access" subtitle="reset password">
            {sent ? (
                <div className="space-y-3">
                    <Notice kind="ok">
                        If an account exists for that email, a reset link is on its way. The link
                        expires shortly.
                    </Notice>
                    <div className="text-center text-[10px] text-fg-dim">
                        <Link to="/login" className="text-accent hover:underline">back to sign in</Link>
                    </div>
                </div>
            ) : (
                <form onSubmit={onSubmit} className="space-y-3">
                    <Field label="email" type="email" autoComplete="email" required value={email}
                        onChange={(e) => setEmail(e.target.value)} />
                    {error && <Notice kind="error">{error}</Notice>}
                    <Button type="submit" disabled={submitting} className="w-full">
                        {submitting ? "sending…" : "send reset link"}
                    </Button>
                    <div className="pt-1 text-center text-[10px] text-fg-dim">
                        <Link to="/login" className="text-accent hover:underline">back to sign in</Link>
                    </div>
                </form>
            )}
        </AuthShell>
    );
}
