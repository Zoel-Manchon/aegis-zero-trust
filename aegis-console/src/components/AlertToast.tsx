/* AlertToast — live WS-alert popups. Portaled to <body> so nothing clips them,
 * stacked (newest on top), auto-dismissed, with a synthesized alert sound.
 * Fires whenever `trigger` (a nonce) changes, so repeat alerts still pop.
 *
 * Anchored bottom-right and painted in solid accent: on a light console this is
 * the only element allowed to shout, so it doesn't need a border or a glow —
 * a block of vermilion on paper is already the loudest thing on screen. */

import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { SecurityAlert } from "@/types";

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

    function dismiss(id: number) {
        setItems((prev) => prev.filter((it) => it.id !== id));
    }

    if (items.length === 0) return null;

    return createPortal(
        <div className="pointer-events-none fixed bottom-4 right-4 z-[9999] flex w-[320px] flex-col gap-2">
            {items.map((it) => (
                <div
                    key={it.id}
                    className="toast-pop pointer-events-auto border-2 border-accent-800 bg-accent text-bg shadow-[0_12px_32px_rgba(45,43,43,0.22)]"
                >
                    <div className="flex items-center justify-between border-b border-accent-800 px-2.5 py-1.5">
                        <span className="micro">
                            {it.alert.severity === "critical" ? "Critical alert" : "Live alert"}
                        </span>
                        <button
                            onClick={() => dismiss(it.id)}
                            aria-label="Dismiss alert"
                            className="cursor-pointer border-0 bg-transparent text-[11px] text-bg"
                        >
                            ✕
                        </button>
                    </div>
                    <div className="p-2.5">
                        <div className="font-heading text-[15px] font-extrabold">{it.alert.title}</div>
                        <div className="mt-1 text-[11px] leading-[1.45]">{it.alert.description}</div>
                    </div>
                </div>
            ))}
        </div>,
        document.body,
    );
}
