/* Passkeys — list, register, and revoke. The browser ceremony runs through
 * navigator.credentials and is verified server-side by webauthn-rs. */

import { useCallback, useEffect, useState } from "react";
import { passkeysApi } from "@/lib/api/passkeys";
import { enrollPasskey, passkeysSupported } from "@/lib/auth/webauthn";
import { Panel } from "@/components/Panel";
import { Notice } from "@/components/Notice";
import type { PasskeyCredentialView } from "@/types";

export function PasskeysSection() {
    const [items, setItems] = useState<PasskeyCredentialView[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [note, setNote] = useState<string | null>(null);
    const [enrolling, setEnrolling] = useState(false);

    const load = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            setItems(await passkeysApi.list());
        } catch {
            setError("Couldn't load passkeys.");
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        void load();
    }, [load]);

    async function remove(id: string) {
        try {
            await passkeysApi.remove(id);
            setItems((prev) => prev.filter((p) => p.credential_id !== id));
        } catch {
            setError("Couldn't revoke that passkey.");
        }
    }

    async function enroll() {
        setNote(null);
        setError(null);
        if (!passkeysSupported()) {
            setError("This browser doesn't support passkeys.");
            return;
        }
        setEnrolling(true);
        try {
            await enrollPasskey("This device");
            setNote("Passkey registered.");
            await load();
        } catch (e) {
            // A user cancelling the native prompt throws; keep it non-alarming.
            const name = e instanceof Error ? e.name : "";
            setError(
                name === "NotAllowedError" ? "Enrollment cancelled." : "Couldn't register that passkey.",
            );
        } finally {
            setEnrolling(false);
        }
    }

    return (
        <Panel
            title="Passkeys"
            right={
                <button onClick={enroll} disabled={enrolling} className="btn btn-ghost micro">
                    {enrolling ? "Waiting for device…" : "+ Register device"}
                </button>
            }
            bodyClassName=""
        >
            {(error || note) && (
                <div className="p-3 pb-0">
                    {error && <Notice kind="error">{error}</Notice>}
                    {note && <Notice kind="info">{note}</Notice>}
                </div>
            )}

            {loading ? (
                <p className="p-3 text-[11px] text-fg-dim">Loading…</p>
            ) : items.length === 0 ? (
                <p className="p-3 text-[12px] leading-[1.6] text-fg-dim">
                    No passkeys yet. Register this device to sign in without a password.
                </p>
            ) : (
                <table className="table text-[12px]">
                    <tbody>
                        {items.map((p) => (
                            <tr key={p.credential_id}>
                                <td className="px-3 py-2 font-heading font-extrabold">
                                    {p.friendly_name ?? "Passkey"}
                                </td>
                                <td className="px-2 py-2 text-fg-dim">
                                    {p.transports.join(" · ") || "—"}
                                </td>
                                <td className="px-2 py-2 text-fg-dim">
                                    added {new Date(p.created_at).toISOString().slice(0, 10)}
                                </td>
                                <td className="px-3 py-2 text-right">
                                    <button
                                        onClick={() => remove(p.credential_id)}
                                        className="btn btn-ghost micro"
                                    >
                                        Remove
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </Panel>
    );
}
