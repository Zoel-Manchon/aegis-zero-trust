/* =============================================================================
 * sse.ts — Server-Sent Events over fetch().
 *
 * The native EventSource cannot send an Authorization header, but the backend's
 * /admin/security/alerts/stream sits behind bearer auth. So we consume the
 * stream with fetch() + a ReadableStream reader, attach the token, parse the
 * `event:` / `data:` frames ourselves, and reconnect with backoff. A 401 mid-
 * stream triggers the same silent-refresh path the api client uses.
 * ========================================================================== */

import { API_BASE, accessToken } from "@/lib/api/client";

export interface SseHandle {
    /** Stop the stream and prevent further reconnects. */
    close: () => void;
}

interface SseOptions<T> {
    /** The SSE `event:` name to listen for (others are ignored). */
    eventName: string;
    onMessage: (data: T) => void;
    onStatus?: (status: "open" | "reconnecting" | "closed") => void;
    /** Called to obtain a fresh token after a 401 (returns true if refreshed). */
    onUnauthorized?: () => Promise<boolean>;
}

export function streamSSE<T>(path: string, opts: SseOptions<T>): SseHandle {
    let closed = false;
    let controller: AbortController | null = null;
    let backoff = 1000;

    const run = async () => {
        while (!closed) {
            controller = new AbortController();
            try {
                const token = accessToken();
                const res = await fetch(`${API_BASE}${path}`, {
                    headers: {
                        Accept: "text/event-stream",
                        ...(token ? { Authorization: `Bearer ${token}` } : {}),
                    },
                    signal: controller.signal,
                });

                if (res.status === 401 && opts.onUnauthorized) {
                    const ok = await opts.onUnauthorized();
                    if (!ok) {
                        opts.onStatus?.("closed");
                        return;
                    }
                    continue; // retry immediately with the new token
                }
                if (!res.ok || !res.body) {
                    throw new Error(`stream failed: ${res.status}`);
                }

                opts.onStatus?.("open");
                backoff = 1000;

                const reader = res.body.getReader();
                const decoder = new TextDecoder();
                let buffer = "";

                // eslint-disable-next-line no-constant-condition
                while (true) {
                    const { done, value } = await reader.read();
                    if (done) break;
                    buffer += decoder.decode(value, { stream: true });

                    let sep: number;
                    while ((sep = buffer.indexOf("\n\n")) !== -1) {
                        const frame = buffer.slice(0, sep);
                        buffer = buffer.slice(sep + 2);
                        dispatch(frame, opts);
                    }
                }
            } catch {
                if (closed) return;
            }

            if (closed) return;
            opts.onStatus?.("reconnecting");
            await sleep(backoff);
            backoff = Math.min(backoff * 2, 15000);
        }
    };

    function dispatch(frame: string, o: SseOptions<T>) {
        let event = "message";
        const dataLines: string[] = [];
        for (const line of frame.split("\n")) {
            if (line.startsWith("event:")) event = line.slice(6).trim();
            else if (line.startsWith("data:")) dataLines.push(line.slice(5).trim());
        }
        if (event !== o.eventName || dataLines.length === 0) return;
        try {
            o.onMessage(JSON.parse(dataLines.join("\n")) as T);
        } catch {
            /* ignore malformed frame */
        }
    }

    void run();

    return {
        close() {
            closed = true;
            controller?.abort();
            opts.onStatus?.("closed");
        },
    };
}

function sleep(ms: number): Promise<void> {
    return new Promise((r) => setTimeout(r, ms));
}
