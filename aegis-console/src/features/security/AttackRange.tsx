/* AttackRange — the SOC's red-team control. Pick an attacker ORIGIN and a
 * SCENARIO, name a victim account, and LAUNCH. Events stream straight into the
 * feed; launching from two distant origins in a row trips impossible-travel. */

import { useEffect, useState } from "react";
import { attackRangeApi } from "@/lib/api/attackRange";
import type { LaunchReport, OriginPreset, ScenarioInfo } from "@/lib/api/attackRange";
import { ApiClientError } from "@/lib/api/client";
import { Panel } from "@/components/Panel";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";

export function AttackRange() {
    const [scenarios, setScenarios] = useState<ScenarioInfo[]>([]);
    const [origins, setOrigins] = useState<OriginPreset[]>([]);
    const [scenario, setScenario] = useState("brute_force");
    const [origin, setOrigin] = useState("madrid");
    const [victim, setVictim] = useState("");
    const [busy, setBusy] = useState(false);
    const [log, setLog] = useState<LaunchReport[]>([]);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        attackRangeApi
            .scenarios()
            .then((r) => {
                setScenarios(r.scenarios);
                setOrigins(r.origins);
                if (r.scenarios[0]) setScenario(r.scenarios[0].key);
                if (r.origins[0]) setOrigin(r.origins[0].key);
            })
            .catch(() => setError("Couldn't load attack scenarios."));
    }, []);

    function mapError(e: unknown): string {
        const code = e instanceof ApiClientError ? e.code : "UNKNOWN";
        return code === "NOT_FOUND" || code === "HTTP_404"
            ? "No account with that email. Register the victim first."
            : code === "AUTH_UNAUTHORIZED" || code === "HTTP_401"
              ? "Admin role required to launch attacks."
              : "Launch failed. Check the backend is running.";
    }

    async function launch() {
        setError(null);
        if (!victim.trim()) {
            setError("Enter a victim account email (a registered user).");
            return;
        }
        setBusy(true);
        try {
            const r = await attackRangeApi.launch({ scenario, origin, victim_email: victim.trim() });
            setLog((prev) => [r, ...prev].slice(0, 12));
        } catch (e) {
            setError(mapError(e));
        } finally {
            setBusy(false);
        }
    }

    // Run every scenario in sequence from the selected origin. Pure client-side
    // orchestration over the single-launch endpoint — each result streams into
    // the log (and the live feed) as it lands.
    async function runAll() {
        setError(null);
        if (!victim.trim()) {
            setError("Enter a victim account email (a registered user).");
            return;
        }
        setBusy(true);
        try {
            for (const s of scenarios) {
                const r = await attackRangeApi.launch({ scenario: s.key, origin, victim_email: victim.trim() });
                setLog((prev) => [r, ...prev].slice(0, 12));
            }
        } catch (e) {
            setError(mapError(e));
        } finally {
            setBusy(false);
        }
    }

    const selectCls =
        "w-full border border-line bg-bg px-2 py-1.5 text-xs text-fg outline-none focus:border-accent";

    return (
        <Panel title="attack range · red team" right="operator console">
            <div className="space-y-3 p-1">
                <div className="grid gap-2 sm:grid-cols-3">
                    <label className="block">
                        <span className="block text-[10px] uppercase tracking-[1.5px] text-fg-dim">attacker origin</span>
                        <select className={`mt-1 ${selectCls}`} value={origin} onChange={(e) => setOrigin(e.target.value)}>
                            {origins.map((o) => (
                                <option key={o.key} value={o.key}>{o.label}</option>
                            ))}
                        </select>
                    </label>
                    <label className="block">
                        <span className="block text-[10px] uppercase tracking-[1.5px] text-fg-dim">scenario</span>
                        <select className={`mt-1 ${selectCls}`} value={scenario} onChange={(e) => setScenario(e.target.value)}>
                            {scenarios.map((s) => (
                                <option key={s.key} value={s.key}>{s.label}</option>
                            ))}
                        </select>
                    </label>
                    <label className="block">
                        <span className="block text-[10px] uppercase tracking-[1.5px] text-fg-dim">victim email</span>
                        <input className={`mt-1 ${selectCls}`} placeholder="test@example.com" value={victim}
                            onChange={(e) => setVictim(e.target.value)} />
                    </label>
                </div>

                {error && <Notice kind="error">{error}</Notice>}

                <div className="flex flex-wrap items-center gap-3">
                    <Button type="button" variant="danger" onClick={launch} disabled={busy}>
                        {busy ? "launching…" : "▶ launch attack"}
                    </Button>
                    <Button type="button" variant="ghost" onClick={runAll} disabled={busy}>
                        ▶▶ run all scenarios
                    </Button>
                    <span className="text-[10px] text-fg-dim">
                        run all fires every scenario from this origin · launch from two distant origins to trip impossible-travel
                    </span>
                </div>

                {log.length > 0 && (
                    <div className="border border-line bg-bg">
                        <div className="border-b border-line px-2.5 py-1 text-[9px] uppercase tracking-[1.5px] text-fg-dim">
                            launch log
                        </div>
                        <div className="max-h-[180px] space-y-px overflow-y-auto p-1 font-mono">
                            {log.map((r, i) => (
                                <div key={i} className={`px-1.5 py-1 text-[11px] ${r.impossible_travel ? "bg-sev-critical/5" : ""}`}>
                                    <span className="text-fg-dim">▶</span>{" "}
                                    <span className="text-fg">{r.scenario}</span> from{" "}
                                    <span className="text-accent">{r.origin.city}, {r.origin.country}</span>{" "}
                                    <span className="text-fg-dim">({r.origin_ip} · {r.events_recorded} ev)</span>
                                    {r.impossible_travel && (
                                        <span className="ml-1 font-bold text-sev-critical">
                                            ⚠ IMPOSSIBLE_TRAVEL{r.from ? ` ${r.from.city}→${r.origin.city}` : ""} · {Math.round(r.speed_kmh)} km/h
                                        </span>
                                    )}
                                </div>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </Panel>
    );
}
