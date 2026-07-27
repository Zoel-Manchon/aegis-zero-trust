/* =============================================================================
 * apiClient — typed fetch wrapper.
 *
 *   1. Prepends the API base (VITE_API_BASE, default "/api" → Vite dev proxy).
 *   2. Injects the Bearer access token from a holder set by AuthContext.
 *   3. Unwraps the `{ data, error }` envelope (or returns raw for /refresh).
 *   4. On 401 for an authed request, runs a single-flight silent refresh and
 *      retries the request once. Network/5xx/parse errors normalize to
 *      ApiClientError.
 *
 * Layering: this module never imports React. AuthContext wires in the token
 * provider and the refresh handler via the setters below.
 *
 * ZERO-TRUST: the access token lives in memory only; a hard refresh loses it
 * and the auth context silently re-mints from the refresh token on boot.
 * ========================================================================== */

import type { ApiError, ApiResponse } from "@/types";

export const API_BASE = import.meta.env.VITE_API_BASE || "/api";

export class ApiClientError extends Error {
    readonly status: number;
    readonly code: string;

    constructor(status: number, code: string, message: string) {
        super(message);
        this.name = "ApiClientError";
        this.status = status;
        this.code = code;
    }

    isUnauthorized(): boolean {
        return this.status === 401;
    }
    /** Backend asks for MFA / step-up mid-session (risk engine). */
    needsStepUp(): boolean {
        return this.code === "AUTH_MFA_REQUIRED" || this.code === "AUTH_STEP_UP_REQUIRED";
    }
}

/* ---------- token + refresh wiring (set by AuthContext) ---------- */

let accessTokenHolder: () => string | null = () => null;
export function setAccessTokenProvider(fn: () => string | null): void {
    accessTokenHolder = fn;
}

let refreshHandler: (() => Promise<boolean>) | null = null;
export function setRefreshHandler(fn: (() => Promise<boolean>) | null): void {
    refreshHandler = fn;
}

/** Single-flight: concurrent 401s share one refresh round-trip. */
let inFlightRefresh: Promise<boolean> | null = null;
function refreshOnce(): Promise<boolean> {
    if (!refreshHandler) return Promise.resolve(false);
    if (!inFlightRefresh) {
        inFlightRefresh = refreshHandler().finally(() => {
            inFlightRefresh = null;
        });
    }
    return inFlightRefresh;
}

export const accessToken = () => accessTokenHolder();

/* ---------- core request ---------- */

interface RequestOptions {
    method?: "GET" | "POST" | "PUT" | "DELETE";
    body?: unknown;
    /** Send the bearer token. Default true. */
    auth?: boolean;
    /** For /refresh: tokens at top level, not wrapped in { data }. */
    rawResponse?: boolean;
    /** Internal: prevents infinite 401→refresh→401 loops. */
    _retried?: boolean;
}

async function request<T>(path: string, opts: RequestOptions = {}): Promise<T> {
    const { method = "GET", body, auth = true, rawResponse = false, _retried = false } = opts;

    const headers: Record<string, string> = { Accept: "application/json" };
    if (body !== undefined) headers["Content-Type"] = "application/json";
    if (auth) {
        const token = accessTokenHolder();
        if (token) headers["Authorization"] = `Bearer ${token}`;
    }

    let res: Response;
    try {
        res = await fetch(`${API_BASE}${path}`, {
            method,
            headers,
            body: body === undefined ? undefined : JSON.stringify(body),
        });
    } catch (e) {
        throw new ApiClientError(
            0,
            "NETWORK_ERROR",
            e instanceof Error ? e.message : "network request failed",
        );
    }

    // 401 on an authed call → try one silent refresh, then retry once.
    if (res.status === 401 && auth && !_retried) {
        const ok = await refreshOnce();
        if (ok) return request<T>(path, { ...opts, _retried: true });
    }

    let parsed: unknown = null;
    const text = await res.text();
    if (text.length > 0) {
        try {
            parsed = JSON.parse(text);
        } catch {
            throw new ApiClientError(res.status, "INVALID_JSON", "server returned non-JSON body");
        }
    }

    if (!res.ok) {
        const err = (parsed as ApiResponse<unknown> | null)?.error as ApiError | undefined;
        throw new ApiClientError(
            res.status,
            err?.code ?? `HTTP_${res.status}`,
            err?.message ?? res.statusText,
        );
    }

    if (rawResponse) return parsed as T;

    const env = parsed as ApiResponse<T> | null;
    if (env?.error) throw new ApiClientError(res.status, env.error.code, env.error.message);
    return (env?.data ?? null) as T;
}

export const api = {
    get: <T>(path: string, opts?: Omit<RequestOptions, "method" | "body" | "_retried">) =>
        request<T>(path, { ...opts, method: "GET" }),
    post: <T>(
        path: string,
        body?: unknown,
        opts?: Omit<RequestOptions, "method" | "body" | "_retried">,
    ) => request<T>(path, { ...opts, method: "POST", body }),
    delete: <T>(
        path: string,
        body?: unknown,
        opts?: Omit<RequestOptions, "method" | "body" | "_retried">,
    ) => request<T>(path, { ...opts, method: "DELETE", body }),
};
