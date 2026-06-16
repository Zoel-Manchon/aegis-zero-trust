/* MFA enrollment & removal. Enrollment shows the TOTP secret + a scannable QR
 * built from the otpauth:// URI, then confirms with the first code. */

import { useEffect, useState, type FormEvent } from "react";
import QRCode from "qrcode";
import { mfaApi } from "@/lib/api/mfa";
import { ApiClientError } from "@/lib/api/client";
import { useAuth } from "@/lib/auth/AuthContext";
import { Panel } from "@/components/Panel";
import { Field } from "@/components/Field";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";
import type { MfaSetupResponse } from "@/types";

export function MfaSection() {
    const { user, refreshUser } = useAuth();
    const [setup, setSetup] = useState<MfaSetupResponse | null>(null);
    const [qrSvg, setQrSvg] = useState<string>("");
    const [code, setCode] = useState("");
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [ok, setOk] = useState<string | null>(null);

    useEffect(() => {
        if (!setup?.otpauth_url) {
            setQrSvg("");
            return;
        }
        let alive = true;
        QRCode.toString(setup.otpauth_url, {
            type: "svg",
            margin: 1,
            width: 180,
            color: { dark: "#c9d4e0", light: "#00000000" },
        })
            .then((svg) => alive && setQrSvg(svg))
            .catch(() => alive && setQrSvg(""));
        return () => {
            alive = false;
        };
    }, [setup]);

    async function begin() {
        setError(null);
        setOk(null);
        setBusy(true);
        try {
            setSetup(await mfaApi.setup());
        } catch {
            setError("Couldn't start MFA setup. Try again.");
        } finally {
            setBusy(false);
        }
    }

    async function confirm(e: FormEvent) {
        e.preventDefault();
        setError(null);
        setBusy(true);
        try {
            await mfaApi.verifySetup({ code: code.trim() });
            setSetup(null);
            setCode("");
            setOk("MFA enabled. You'll be prompted for a code at sign-in.");
            await refreshUser();
        } catch (err) {
            const c = err instanceof ApiClientError ? err.code : "UNKNOWN";
            setError(c === "AUTH_UNAUTHORIZED" ? "That code didn't verify." : "Couldn't enable MFA.");
        } finally {
            setBusy(false);
        }
    }

    async function disable(e: FormEvent) {
        e.preventDefault();
        setError(null);
        setBusy(true);
        try {
            await mfaApi.disable({ code: code.trim() });
            setCode("");
            setOk("MFA disabled.");
            await refreshUser();
        } catch {
            setError("Couldn't disable MFA — check the code.");
        } finally {
            setBusy(false);
        }
    }

    const enabled = user?.mfa_enabled ?? false;

    return (
        <Panel title="multi-factor authentication" right={enabled ? "enabled" : "disabled"}>
            <div className="space-y-3 p-1 text-[11px]">
                {ok && <Notice kind="ok">{ok}</Notice>}
                {error && <Notice kind="error">{error}</Notice>}

                {enabled ? (
                    <form onSubmit={disable} className="space-y-2">
                        <p className="text-fg-dim">
                            Time-based one-time codes are active on this account. Enter a current code
                            to turn MFA off.
                        </p>
                        <Field label="current code" inputMode="numeric" maxLength={6} value={code}
                            onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))} />
                        <Button type="submit" variant="danger" disabled={busy || code.length !== 6}>
                            {busy ? "working…" : "disable mfa"}
                        </Button>
                    </form>
                ) : setup ? (
                    <form onSubmit={confirm} className="space-y-3">
                        <p className="text-fg-dim">
                            Scan this in your authenticator app, or enter the secret manually, then
                            confirm with the first code.
                        </p>
                        {qrSvg && (
                            <div
                                className="mx-auto w-[180px] border border-line bg-bg p-2"
                                // QR is generated locally from the otpauth URI; no external calls.
                                dangerouslySetInnerHTML={{ __html: qrSvg }}
                            />
                        )}
                        <div className="break-all border border-line bg-bg px-2 py-1.5 text-[10px] text-fg-dim">
                            secret: <span className="text-fg">{setup.secret}</span>
                        </div>
                        <Field label="first code" inputMode="numeric" maxLength={6} value={code}
                            onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))} />
                        <div className="flex gap-2">
                            <Button type="submit" disabled={busy || code.length !== 6}>
                                {busy ? "verifying…" : "confirm & enable"}
                            </Button>
                            <Button type="button" variant="ghost" onClick={() => setSetup(null)}>
                                cancel
                            </Button>
                        </div>
                    </form>
                ) : (
                    <div className="space-y-2">
                        <p className="text-fg-dim">
                            Add a second factor (TOTP). Strongly recommended for any account that can
                            reach the admin console.
                        </p>
                        <Button type="button" onClick={begin} disabled={busy}>
                            {busy ? "starting…" : "set up mfa"}
                        </Button>
                    </div>
                )}
            </div>
        </Panel>
    );
}
