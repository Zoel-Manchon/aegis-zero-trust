/* =============================================================================
 * jwt.ts — decode (NOT verify) a JWT payload, browser-side.
 *
 * We never trust this for security decisions — the Rust middleware verifies the
 * signature on every request. We decode purely to read claims the client needs
 * operationally: the `jti` (so the silent-refresh loop can chain across the
 * backend's rotating sessions) and `exp` (for pre-emptive refresh scheduling).
 * ========================================================================== */

export interface AccessClaims {
    sub: number;
    jti: string;
    purpose: string;
    exp: number;
    iat: number;
}

function base64UrlDecode(segment: string): string {
    const padded = segment.replace(/-/g, "+").replace(/_/g, "/");
    const withPad = padded + "=".repeat((4 - (padded.length % 4)) % 4);
    // decodeURIComponent dance handles any UTF-8 in claims safely.
    return decodeURIComponent(
        atob(withPad)
            .split("")
            .map((c) => "%" + c.charCodeAt(0).toString(16).padStart(2, "0"))
            .join(""),
    );
}

/** Returns the decoded claims, or null if the token is malformed. */
export function decodeAccessToken(token: string): AccessClaims | null {
    const parts = token.split(".");
    if (parts.length !== 3) return null;
    try {
        const payload = JSON.parse(base64UrlDecode(parts[1])) as Partial<AccessClaims>;
        if (typeof payload.jti !== "string" || typeof payload.exp !== "number") {
            return null;
        }
        return payload as AccessClaims;
    } catch {
        return null;
    }
}

/** The session jti baked into an access token, or null. */
export function jtiFromToken(token: string): string | null {
    return decodeAccessToken(token)?.jti ?? null;
}

/** Seconds until the token expires (negative if already expired), or null. */
export function secondsUntilExpiry(token: string): number | null {
    const exp = decodeAccessToken(token)?.exp;
    if (exp == null) return null;
    return exp - Math.floor(Date.now() / 1000);
}
