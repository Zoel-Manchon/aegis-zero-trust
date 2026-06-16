/* Passkeys — list and revoke are fully wired. Enrollment is flagged
 * experimental because the backend WebAuthn verification is still stubbed. */

import { useCallback, useEffect, useState } from "react";
import { passkeysApi } from "@/lib/api/passkeys";
import { Panel } from "@/components/Panel";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";
import type { PasskeyCredentialView } from "@/types";

export function PasskeysSection() {
    const [items, setItems] = useState<PasskeyCredentialView[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [note, setNote] = useState<string | null>(null);

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
        try {
            await passkeysApi.registerBegin("This device");
            setNote(
                "Enrollment challenge issued. Full WebAuthn enrollment is experimental — the backend verifier isn't wired yet.",
            );
        } catch {
            setError("Couldn't begin enrollment.");
        }
    }

    return (
        <Panel title="passkeys" right={`${items.length} registered`}>
            <div className="space-y-3 p-1 text-[11px]">
                {error && <Notice kind="error">{error}</Notice>}
                {note && <Notice kind="info">{note}</Notice>}

                {loading ? (
                    <p className="text-fg-dim">loading…</p>
                ) : items.length === 0 ? (
                    <p className="text-fg-dim">No passkeys registered.</p>
                ) : (
                    <table className="w-full border-collapse">
                        <tbody>
                            {items.map((p) => (
                                <tr key={p.credential_id} className="border-b border-grid">
                                    <td className="py-1.5 pr-2 text-fg">{p.friendly_name ?? "passkey"}</td>
                                    <td className="py-1.5 pr-2 text-fg-dim">
                                        {p.transports.join(", ") || "—"}
                                    </td>
                                    <td className="py-1.5 pr-2 text-fg-dim">
                                        {new Date(p.created_at).toISOString().slice(0, 10)}
                                    </td>
                                    <td className="py-1.5 text-right">
                                        <button
                                            onClick={() => remove(p.credential_id)}
                                            className="text-[10px] uppercase text-sev-high hover:underline"
                                        >
                                            revoke
                                        </button>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}

                <Button type="button" variant="ghost" onClick={enroll}>
                    add passkey (experimental)
                </Button>
            </div>
        </Panel>
    );
}
