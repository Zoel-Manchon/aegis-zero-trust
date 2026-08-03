/* MFA enrollment & removal. Enrollment shows the TOTP secret + a scannable QR
 * built from the otpauth:// URI, then confirms with the first code.
 *
 * QR on the left at a fixed 150px, instructions and the code field on the right:
 * the two halves of one task, side by side, so nobody scrolls mid-enrollment. */

import { useEffect, useState, type FormEvent } from "react";
import QRCode from "qrcode";
import { mfaApi } from "@/lib/api/mfa";
import { ApiClientError } from "@/lib/api/client";
import { useAuth } from "@/lib/auth/AuthContext";
import { Panel } from "@/components/Panel";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";
import type { MfaSetupResponse } from "@/types";

/** Group the base32 secret in fours — it's transcribed by hand often enough
 *  that the grouping earns its place. */
function grouped(secret: string): string {
    return secret.replace(/(.{4})/g, "$1 ").trim();
}

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
            width: 150,
            color: { dark: "#201e1d", light: "#00000000" },
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
        <Panel
            title="Two-factor authentication"
            right={
                <span className={enabled ? "text-accent-700" : "text-fg-dim"}>
                    {enabled ? "Enabled" : "Not enrolled"}
                </span>
            }
            rule
            bodyClassName="p-3.5"
        >
            <div className="space-y-3">
                {ok && <Notice kind="ok">{ok}</Notice>}
                {error && <Notice kind="error">{error}</Notice>}

                {enabled ? (
                    <form onSubmit={disable} className="space-y-3">
                        <p className="max-w-[60ch] text-[12px] leading-[1.6] text-neutral-800">
                            Time-based one-time codes are active on this account. Enter a current
                            code to turn them off. Admin access to the operations console requires
                            this factor.
                        </p>
                        <div className="flex flex-wrap items-end gap-2.5">
                            <div className="field w-[180px]">
                                <label htmlFor="mfa-off">Verification code</label>
                                <input
                                    id="mfa-off"
                                    className="input tracking-[0.3em]"
                                    inputMode="numeric"
                                    maxLength={6}
                                    placeholder="000000"
                                    value={code}
                                    onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                                />
                            </div>
                            <Button type="submit" disabled={busy || code.length !== 6}>
                                {busy ? "Working…" : "Disable"}
                            </Button>
                        </div>
                    </form>
                ) : setup ? (
                    <form onSubmit={confirm} className="grid gap-4.5 sm:grid-cols-[150px_1fr]">
                        <div
                            className="h-[150px] w-[150px] bg-panel-hi p-2.5"
                            // QR is generated locally from the otpauth URI; no external calls.
                            dangerouslySetInnerHTML={{ __html: qrSvg }}
                        />
                        <div>
                            <p className="max-w-[60ch] text-[12px] leading-[1.6] text-neutral-800">
                                Scan the code with a TOTP authenticator, then confirm a six-digit
                                code. Admin access to the operations console is gated behind this
                                factor.
                            </p>
                            <div className="mt-3 text-[11px] uppercase tracking-[0.1em] text-fg-dim">
                                Secret
                            </div>
                            <div className="break-all font-heading text-[15px] font-extrabold tracking-[0.18em]">
                                {grouped(setup.secret)}
                            </div>
                            <div className="mt-3.5 flex flex-wrap items-end gap-2.5">
                                <div className="field w-[180px]">
                                    <label htmlFor="mfa-code">Verification code</label>
                                    <input
                                        id="mfa-code"
                                        className="input tracking-[0.3em]"
                                        inputMode="numeric"
                                        maxLength={6}
                                        placeholder="000000"
                                        value={code}
                                        onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))}
                                    />
                                </div>
                                <Button type="submit" disabled={busy || code.length !== 6}>
                                    {busy ? "Verifying…" : "Verify & enable"}
                                </Button>
                                <Button type="button" variant="secondary" onClick={() => setSetup(null)}>
                                    Cancel
                                </Button>
                            </div>
                        </div>
                    </form>
                ) : (
                    <div className="space-y-3">
                        <p className="max-w-[60ch] text-[12px] leading-[1.6] text-neutral-800">
                            Add a second factor (TOTP). Required for any account that can reach the
                            operations console.
                        </p>
                        <Button type="button" onClick={begin} disabled={busy}>
                            {busy ? "Starting…" : "Set up MFA"}
                        </Button>
                    </div>
                )}
            </div>
        </Panel>
    );
}
