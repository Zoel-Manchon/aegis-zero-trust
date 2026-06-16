/* VerifyEmailPage — two modes:
 *  - ?token=...  → auto-confirm the address on load.
 *  - no token    → request a verification email by address. */

import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { verifyEmailApi } from "@/lib/api/auth";
import { ApiClientError } from "@/lib/api/client";
import { AuthShell } from "@/components/AuthShell";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

type ConfirmState = "pending" | "ok" | "error";

export default function VerifyEmailPage() {
    const [params] = useSearchParams();
    const token = useMemo(() => params.get("token") ?? "", [params]);

    const [email, setEmail] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [requested, setRequested] = useState(false);
    const [reqError, setReqError] = useState<string | null>(null);
    const [confirm, setConfirm] = useState<ConfirmState>("pending");
    const ran = useRef(false);

    useEffect(() => {
        if (!token || ran.current) return;
        ran.current = true;
        (async () => {
            try {
                await verifyEmailApi.confirm({ token });
                setConfirm("ok");
            } catch {
                setConfirm("error");
            }
        })();
    }, [token]);

    async function onRequest(e: FormEvent) {
        e.preventDefault();
        setReqError(null);
        setSubmitting(true);
        try {
            await verifyEmailApi.request({ email: email.trim() });
        } catch (err) {
            if (err instanceof ApiClientError && err.code === "NETWORK_ERROR") {
                setReqError("Can't reach the server.");
                setSubmitting(false);
                return;
            }
        }
        setRequested(true);
        setSubmitting(false);
    }

    if (token) {
        return (
            <AuthShell barLabel="verify email" subtitle="confirming address">
                {confirm === "pending" && <Notice kind="info">Confirming your email…</Notice>}
                {confirm === "ok" && (
                    <div className="space-y-3">
                        <Notice kind="ok">Email verified. You're all set.</Notice>
                        <div className="text-center text-[10px] text-fg-dim">
                            <Link to="/login" className="text-accent hover:underline">continue to sign in</Link>
                        </div>
                    </div>
                )}
                {confirm === "error" && (
                    <div className="space-y-3">
                        <Notice kind="error">This verification link is invalid or has expired.</Notice>
                        <div className="text-center text-[10px] text-fg-dim">
                            <Link to="/verify-email" className="text-accent hover:underline">request a new link</Link>
                        </div>
                    </div>
                )}
            </AuthShell>
        );
    }

    return (
        <AuthShell barLabel="verify email" subtitle="request a verification link">
            {requested ? (
                <Notice kind="ok">
                    If that address needs verification, a link is on its way.
                </Notice>
            ) : (
                <form onSubmit={onRequest} className="space-y-3">
                    <Field label="email" type="email" autoComplete="email" required value={email}
                        onChange={(e) => setEmail(e.target.value)} />
                    {reqError && <Notice kind="error">{reqError}</Notice>}
                    <Button type="submit" disabled={submitting} className="w-full">
                        {submitting ? "sending…" : "send verification link"}
                    </Button>
                    <div className="pt-1 text-center text-[10px] text-fg-dim">
                        <Link to="/login" className="text-accent hover:underline">back to sign in</Link>
                    </div>
                </form>
            )}
        </AuthShell>
    );
}
