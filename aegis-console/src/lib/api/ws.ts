import { API_BASE, accessToken } from "@/lib/api/client";

export interface WsHandle { close: () => void }

interface WsOptions<T> {
    onMessage: (data: T) => void;
    onStatus?: (status: "open" | "reconnecting" | "closed") => void;
    onUnauthorized?: () => Promise<boolean>;
}

function wsUrl(path: string): string {
    const base = API_BASE.startsWith("http") ? API_BASE : `${window.location.origin}${API_BASE}`;
    const url = new URL(`${base}${path}`);
    url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
    const token = accessToken();
    if (token) url.searchParams.set("access_token", token);
    return url.toString();
}

export function streamWS<T>(path: string, opts: WsOptions<T>): WsHandle {
    let closed = false;
    let socket: WebSocket | null = null;
    let backoff = 1000;

    const connect = () => {
        if (closed) return;
        socket = new WebSocket(wsUrl(path));
        socket.onopen = () => { backoff = 1000; opts.onStatus?.("open"); };
        socket.onmessage = (ev) => {
            try { opts.onMessage(JSON.parse(String(ev.data)) as T); } catch { /* ignore */ }
        };
        socket.onclose = async (ev) => {
            if (closed) return;
            opts.onStatus?.("reconnecting");
            // Policy close/unauthorized path: try one token refresh before reconnecting.
            if ((ev.code === 1008 || ev.code === 4001) && opts.onUnauthorized) {
                const ok = await opts.onUnauthorized();
                if (!ok) { opts.onStatus?.("closed"); return; }
            }
            window.setTimeout(connect, backoff);
            backoff = Math.min(backoff * 2, 15000);
        };
        socket.onerror = () => socket?.close();
    };

    connect();
    return { close() { closed = true; socket?.close(); opts.onStatus?.("closed"); } };
}
