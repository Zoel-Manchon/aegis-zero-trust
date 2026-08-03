/* AttackRange — the SOC's red-team control. Pick an attacker ORIGIN and a
 * SCENARIO, name a victim account, and LAUNCH. Events stream straight into the
 * console feed; launching from two distant origins in a row trips
 * impossible-travel.
 *
 * Two columns: the console and the scenario library on the left, the launch log
 * on the right. The log is the point of the screen — it's the receipt that says
 * what you fired, from where, and whether the detector caught it. */

import { useEffect, useState } from "react";
import { attackRangeApi } from "@/lib/api/attackRange";
import type { LaunchReport, OriginPreset, ScenarioInfo } from "@/lib/api/attackRange";
import { ApiClientError } from "@/lib/api/client";
import { Panel } from "@/components/Panel";
import { Button } from "@/components/Button";
import { Notice } from "@/components/Notice";
import { sevChip } from "@/components/severity";
import type { Severity } from "@/types";

/* Peak severity per scenario — mirrors scenario_events() in
 * aegis-api/src/modules/attack_range/application/attack_range_service.rs.
 * The /scenarios endpoint doesn't publish it, so keep the two in step by hand. */
const PEAK_SEVERITY: Record<string, Severity> = {
    brute_force: "high",
    credential_stuffing: "high",
    token_replay: "critical",
    jwt_tamper: "high",
    fingerprint_spoof: "high",
    session_hijack: "critical",
    mfa_bypass: "high",
    rbac_bypass: "high",
    privilege_escalation: "critical",
    storm: "critical",
};

interface LogRow extends LaunchReport {
    at: string;
}

export function AttackRange() {
    const [scenarios, setScenarios] = useState<ScenarioInfo[]>([]);
    const [origins, setOrigins] = useState<OriginPreset[]>([]);
    const [scenario, setScenario] = useState("brute_force");
    const [origin, setOrigin] = useState("madrid");
    const [victim, setVictim] = useState("");
    const [busy, setBusy] = useState(false);
    const [log, setLog] = useState<LogRow[]>([]);
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

    function record(r: LaunchReport) {
        setLog((prev) =>
            [{ ...r, at: new Date().toISOString().slice(11, 19) }, ...prev].slice(0, 20),
        );
    }

    async function launch() {
        setError(null);
        if (!victim.trim()) {
            setError("Enter a victim account email (a registered user).");
            return;
        }
        setBusy(true);
        try {
            record(await attackRangeApi.launch({ scenario, origin, victim_email: victim.trim() }));
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
                record(
                    await attackRangeApi.launch({
                        scenario: s.key,
                        origin,
                        victim_email: victim.trim(),
                    }),
                );
            }
        } catch (e) {
            setError(mapError(e));
        } finally {
            setBusy(false);
        }
    }

    return (
        <div className="grid items-start gap-3 p-4 lg:grid-cols-[1.1fr_1fr]">
            <div className="flex flex-col gap-3">
                <Panel
                    title={<span className="text-accent-700">Attack range · red team</span>}
                    right="operator console"
                    rule
                    bodyClassName="p-3.5"
                >
                    <div className="grid gap-3 sm:grid-cols-2">
                        <div className="field">
                            <label htmlFor="rg-origin">Attacker origin</label>
                            <select
                                id="rg-origin"
                                className="input"
                                value={origin}
                                onChange={(e) => setOrigin(e.target.value)}
                            >
                                {origins.map((o) => (
                                    <option key={o.key} value={o.key}>
                                        {o.label}
                                    </option>
                                ))}
                            </select>
                        </div>
                        <div className="field">
                            <label htmlFor="rg-scenario">Scenario</label>
                            <select
                                id="rg-scenario"
                                className="input"
                                value={scenario}
                                onChange={(e) => setScenario(e.target.value)}
                            >
                                {scenarios.map((s) => (
                                    <option key={s.key} value={s.key}>
                                        {s.label}
                                    </option>
                                ))}
                            </select>
                        </div>
                    </div>

                    <div className="field mt-3">
                        <label htmlFor="rg-victim">Victim account</label>
                        <input
                            id="rg-victim"
                            className="input"
                            placeholder="victim@test.com"
                            value={victim}
                            onChange={(e) => setVictim(e.target.value)}
                        />
                    </div>

                    {error && (
                        <div className="mt-3">
                            <Notice kind="error">{error}</Notice>
                        </div>
                    )}

                    <div className="mt-3.5 flex flex-wrap gap-2.5">
                        <Button type="button" onClick={launch} disabled={busy}>
                            {busy ? "Launching…" : "▶ Launch attack"}
                        </Button>
                        <Button type="button" variant="secondary" onClick={runAll} disabled={busy}>
                            ▶▶ Run all scenarios
                        </Button>
                    </div>
                    <p className="mt-2.5 max-w-[62ch] text-[11px] leading-[1.5] text-fg-dim">
                        Run all fires every scenario from the selected origin. Launch twice from
                        distant origins to trip the impossible-travel detector.
                    </p>
                </Panel>

                <Panel title="Scenario library" bodyClassName="">
                    <table className="table text-[11px]">
                        <thead>
                            <tr>
                                <th className="px-3 py-1.5 text-left text-[10px] uppercase tracking-[0.14em]">
                                    Scenario
                                </th>
                                <th className="px-2 py-1.5 text-left text-[10px] uppercase tracking-[0.14em]">
                                    Emits
                                </th>
                                <th className="px-3 py-1.5 text-right text-[10px] uppercase tracking-[0.14em]">
                                    Severity
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {scenarios.map((s) => {
                                const sev = PEAK_SEVERITY[s.key] ?? "medium";
                                return (
                                    <tr key={s.key}>
                                        <td className="px-3 py-1.5 font-heading font-extrabold">
                                            {s.label}
                                        </td>
                                        <td className="px-2 py-1.5 text-fg-dim">{s.description}</td>
                                        <td className="px-3 py-1.5 text-right">
                                            <span
                                                className={`px-1.5 py-0.5 text-[9px] uppercase tracking-[0.14em] ${sevChip[sev]}`}
                                            >
                                                {sev}
                                            </span>
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </Panel>
            </div>

            <Panel title="Launch log" right={`${log.length} runs`} rule bodyClassName="">
                <div className="max-h-[620px] overflow-y-auto">
                    {log.length === 0 ? (
                        <div className="px-3 py-4 text-[11px] leading-[1.5] text-fg-dim">
                            No launches yet. Pick an origin and a scenario, then launch — events land
                            in the console feed immediately.
                        </div>
                    ) : (
                        log.map((r, i) => (
                            <div
                                key={`${r.scenario}-${i}`}
                                className={`border-b border-neutral-200 px-3 py-2.5 ${
                                    r.impossible_travel ? "bg-accent-100" : ""
                                }`}
                            >
                                <div className="flex justify-between gap-2.5 text-[11px]">
                                    <span className="font-heading font-extrabold uppercase tracking-[0.04em]">
                                        {r.scenario.replace(/_/g, " ")}
                                    </span>
                                    <span className="text-fg-dim">{r.at}</span>
                                </div>
                                <div className="mt-1 text-[11px] text-fg-dim">
                                    {r.origin.city}, {r.origin.country} · {r.origin_ip} ·{" "}
                                    {r.events_recorded} events recorded
                                </div>
                                {r.impossible_travel && (
                                    <div className="mt-1.5 inline-block bg-accent px-1.5 py-0.5 text-[10px] uppercase tracking-[0.12em] text-bg">
                                        ⚠ Impossible travel{" "}
                                        {r.from ? `${r.from.city} → ${r.origin.city} · ` : ""}
                                        {Math.round(r.speed_kmh).toLocaleString()} km/h
                                    </div>
                                )}
                            </div>
                        ))
                    )}
                </div>
            </Panel>
        </div>
    );
}
