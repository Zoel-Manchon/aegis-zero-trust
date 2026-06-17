/* =============================================================================
 * AuthContext — single source of truth for the SPA's auth state.
 *
 * ZERO-TRUST POSTURE:
 *   - Access token in REACT STATE ONLY (memory). Never persisted.
 *   - Refresh token + session jti in sessionStorage (cleared on tab close).
 *   - Token injected into requests via the api client's closure; the client
 *     never imports React.
 *
 * THE REFRESH-CHAIN FIX (and why it bit twice):
 *   The backend ROTATES the session jti on every refresh and treats a reused
 *   (rotated) jti as REPLAY — revoking the whole family. So two problems must
 *   both be solved:
 *     1. After every refresh, store the NEW jti (from the response, or decoded
 *        from the new access token for older backends).
 *     2. NEVER let two refreshes run concurrently. Boot, the SSE stream, and
 *        the api client's 401 retry can all fire at once (React StrictMode
 *        double-invokes effects in dev, too). If two reads grab the same jti
 *        before either writes back, the second is a replay. We coalesce every
 *        caller into ONE in-flight promise.
 * ========================================================================== */

import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useMemo,
    useRef,
    useState,
    type ReactNode,
} from "react";
import { setAccessTokenProvider, setRefreshHandler } from "@/lib/api/client";
import { authApi } from "@/lib/api/auth";
import { jtiFromToken } from "@/lib/auth/jwt";
import { assertPasskey } from "@/lib/auth/webauthn";
import type { LoginResponse, MeResponse, UserRole } from "@/types";

const REFRESH_TOKEN_KEY = "zt.rt";
const JTI_KEY = "zt.jti";

interface AuthState {
    accessToken: string | null;
    booting: boolean;
}

interface AuthContextValue extends AuthState {
    user: MeResponse | null;
    role: UserRole | null;
    /** True once /me has been attempted at least once for the current session. */
    roleResolved: boolean;
    login: (email: string, password: string) => Promise<LoginResponse>;
    loginWithPasskey: (email: string) => Promise<LoginResponse>;
    completeMfa: (mfaToken: string, code: string) => Promise<LoginResponse>;
    logout: () => Promise<void>;
    logoutEverywhere: () => Promise<void>;
    refresh: () => Promise<boolean>;
    refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
    const [state, setState] = useState<AuthState>({ accessToken: null, booting: true });
    const [user, setUser] = useState<MeResponse | null>(null);
    const [roleResolved, setRoleResolved] = useState(false);

    const tokenRef = useRef<string | null>(null);
    tokenRef.current = state.accessToken;

    const storeRotatedJti = useCallback((accessTok: string, explicitJti?: string | null) => {
        const jti = explicitJti ?? jtiFromToken(accessTok);
        if (jti) sessionStorage.setItem(JTI_KEY, jti);
    }, []);

    const clearSession = useCallback(() => {
        sessionStorage.removeItem(REFRESH_TOKEN_KEY);
        sessionStorage.removeItem(JTI_KEY);
        tokenRef.current = null;
        setUser(null);
        setRoleResolved(false);
        setState({ accessToken: null, booting: false });
    }, []);

    const loadMe = useCallback(async () => {
        try {
            setUser(await authApi.me());
        } catch {
            // Older backend without /me, or a transient error: role-unknown.
            setUser(null);
        } finally {
            setRoleResolved(true);
        }
    }, []);

    const applyLoginSuccess = useCallback(
        (res: LoginResponse) => {
            if (!res.access_token || !res.refresh_token) {
                throw new Error("login response missing tokens");
            }
            sessionStorage.setItem(REFRESH_TOKEN_KEY, res.refresh_token);
            storeRotatedJti(res.access_token, res.jti);
            // React state updates asynchronously. Keep the token ref in sync
            // immediately so the following /me request carries Authorization.
            tokenRef.current = res.access_token;
            setRoleResolved(false);
            setState({ accessToken: res.access_token, booting: false });
            void loadMe();
        },
        [storeRotatedJti, loadMe],
    );

    /* ---------- single-flight refresh ---------- */

    const refreshInFlight = useRef<Promise<boolean> | null>(null);

    const doRefresh = useCallback(async () => {
        const rt = sessionStorage.getItem(REFRESH_TOKEN_KEY);
        const jti = sessionStorage.getItem(JTI_KEY);
        if (!rt || !jti) return false;
        try {
            const res = await authApi.refresh({ refresh_token: rt, jti });
            sessionStorage.setItem(REFRESH_TOKEN_KEY, res.refresh_token);
            // Rotate stored jti to match the new session, or the NEXT refresh
            // would replay and nuke the family.
            storeRotatedJti(res.access_token, res.jti);
            // Same immediate-ref sync as login: callers may invoke /me or an
            // admin request before React commits the new state.
            tokenRef.current = res.access_token;
            setState({ accessToken: res.access_token, booting: false });
            return true;
        } catch {
            clearSession();
            return false;
        }
    }, [clearSession, storeRotatedJti]);

    const refresh = useCallback(() => {
        if (!refreshInFlight.current) {
            refreshInFlight.current = doRefresh().finally(() => {
                refreshInFlight.current = null;
            });
        }
        return refreshInFlight.current;
    }, [doRefresh]);

    /* ---------- public API ---------- */

    const login = useCallback(
        async (email: string, password: string) => {
            const res = await authApi.login({ email, password });
            if (!res.mfa_required) applyLoginSuccess(res);
            return res;
        },
        [applyLoginSuccess],
    );

    const loginWithPasskey = useCallback(
        async (email: string) => {
            const res = await assertPasskey(email.trim());
            applyLoginSuccess(res);
            return res;
        },
        [applyLoginSuccess],
    );

    const completeMfa = useCallback(
        async (mfaToken: string, code: string) => {
            const res = await authApi.completeMfa({ mfa_token: mfaToken, code });
            applyLoginSuccess(res);
            return res;
        },
        [applyLoginSuccess],
    );

    const logout = useCallback(async () => {
        try {
            await authApi.logout();
        } catch {
            /* token may already be invalid */
        }
        clearSession();
    }, [clearSession]);

    const logoutEverywhere = useCallback(async () => {
        try {
            await authApi.logoutAll();
        } catch {
            /* best effort */
        }
        clearSession();
    }, [clearSession]);

    const refreshUser = useCallback(() => loadMe(), [loadMe]);

    /* ---------- wiring: token provider + 401 refresh handler ---------- */

    useEffect(() => {
        setAccessTokenProvider(() => tokenRef.current);
        setRefreshHandler(() => refresh());
        return () => setRefreshHandler(null);
    }, [refresh]);

    /* ---------- boot: silent refresh, then load identity ---------- */

    useEffect(() => {
        let cancelled = false;
        (async () => {
            const rt = sessionStorage.getItem(REFRESH_TOKEN_KEY);
            if (!rt) {
                if (!cancelled) setState((s) => ({ ...s, booting: false }));
                return;
            }
            const ok = await refresh();
            if (ok && !cancelled) await loadMe();
        })();
        return () => {
            cancelled = true;
        };
    }, [refresh, loadMe]);

    const value = useMemo<AuthContextValue>(
        () => ({
            ...state,
            user,
            role: user?.role ?? null,
            roleResolved,
            login,
            loginWithPasskey,
            completeMfa,
            logout,
            logoutEverywhere,
            refresh,
            refreshUser,
        }),
        [state, user, roleResolved, login, loginWithPasskey, completeMfa, logout, logoutEverywhere, refresh, refreshUser],
    );

    return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
    const ctx = useContext(AuthContext);
    if (!ctx) throw new Error("useAuth must be used inside <AuthProvider>");
    return ctx;
}
