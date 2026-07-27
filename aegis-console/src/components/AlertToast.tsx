/* AlertToast — live WS-alert popups. Portaled to <body> so nothing clips them,
 * stacked (newest on top), auto-dismissed, with a synthesized alert sound.
 * Fires whenever `trigger` (a nonce) changes, so repeat alerts still pop. */

import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { SecurityAlert } from "@/types";
import { sevText } from "@/components/severity";
import { SevDot } from "@/components/SevDot";

interface ToastItem { id: number; alert: SecurityAlert }

let audio: AudioContext | null = null;
function chime(critical: boolean) {
    try {
        const Ctx =
            window.AudioContext ??
            (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
        if (!Ctx) return;
        audio = audio ?? new Ctx();
        if (audio.state === "suspended") void audio.resume();
        const ctx = audio;
        const now = ctx.currentTime;
        const tones = critical ? [880, 1245, 880] : [659, 880];
        tones.forEach((f, i) => {
            const osc = ctx.createOscillator();
            const gain = ctx.createGain();
            osc.type = "square";
            osc.frequency.value = f;
            const t0 = now + i * 0.13;
            gain.gain.setValueAtTime(0.0001, t0);
            gain.gain.exponentialRampToValueAtTime(0.11, t0 + 0.02);
            gain.gain.exponentialRampToValueAtTime(0.0001, t0 + 0.12);
            osc.connect(gain).connect(ctx.destination);
            osc.start(t0);
            osc.stop(t0 + 0.13);
        });
    } catch {
        /* audio unavailable — popups still work */
    }
}

export function AlertToast({
    alert,
    trigger,
    muted = false,
}: {
    alert: SecurityAlert | null;
    trigger: number;
    muted?: boolean;
}) {
    const [items, setItems] = useState<ToastItem[]>([]);
    const seen = useRef(0);

    useEffect(() => {
        if (!alert || trigger === seen.current) return;
        seen.current = trigger;
        const id = trigger;
        setItems((prev) => [{ id, alert }, ...prev].slice(0, 4));
        if (!muted) chime(alert.severity === "critical");
        const t = window.setTimeout(
            () => setItems((prev) => prev.filter((it) => it.id !== id)),
            7000,
        );
        return () => window.clearTimeout(t);
    }, [trigger, alert, muted]);

    if (items.length === 0) return null;

    return createPortal(
        <div className="pointer-events-none fixed right-4 top-4 z-[9999] flex w-[340px] flex-col gap-2">
            {items.map((it) => {
                const crit = it.alert.severity === "critical";
                return (
                    <div
                        key={it.id}
                        className={`pointer-events-auto border bg-panel/95 shadow-xl backdrop-blur ${crit ? "toast-crit border-sev-critical" : "toast-pop border-sev-high/60"}`}
                    >
                        <div className="flex items-center gap-2 border-b border-line bg-panel-hi px-2.5 py-1.5">
                            <SevDot sev={it.alert.severity} />
                            <span className="text-[10px] font-semibold uppercase tracking-[1.5px] text-fg-dim">
                                live alert
                            </span>
                            <span className={`ml-auto text-[9px] uppercase ${sevText[it.alert.severity]}`}>
                                {it.alert.severity}
                            </span>
                        </div>
                        <div className="space-y-1 p-2.5">
                            <div className={`text-[12px] font-bold ${sevText[it.alert.severity]}`}>
                                {it.alert.title}
                            </div>
                            <div className="text-[10px] leading-snug text-fg-dim">{it.alert.description}</div>
                        </div>
                    </div>
                );
            })}
        </div>,
        document.body,
    );
}
