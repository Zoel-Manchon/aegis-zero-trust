import type { UserRole } from "@/types/common";

export interface RegisterRequest {
    email: string;
    password: string;
}

export interface LoginRequest {
    email: string;
    password: string;
}

/**
 * /login response — discriminated by `mfa_required`.
 *  - success:       mfa_required=false, full token set.
 *  - mfa challenge: mfa_required=true, only `mfa_token`. Exchange it at
 *                   /mfa/complete-login with a TOTP code.
 */
export interface LoginResponse {
    mfa_required?: boolean;
    access_token?: string;
    refresh_token?: string;
    /** access token's jti; used by /refresh */
    jti?: string;
    /** short-lived MFA-purpose token */
    mfa_token?: string;
}

export interface RefreshRequest {
    refresh_token: string;
    jti: string;
}

/** /refresh returns tokens at TOP LEVEL (no envelope). Now includes the
 *  rotated jti (backend patch) — client falls back to decoding the access
 *  token if an older backend omits it. */
export interface RefreshResponse {
    access_token: string;
    refresh_token: string;
    jti?: string;
}

export interface MfaCompleteRequest {
    mfa_token: string;
    code: string;
}

/** GET /me — identity + role + mfa status for the current principal. */
export interface MeResponse {
    user_id: number;
    email: string;
    role: UserRole;
    mfa_enabled: boolean;
    risk_score: number;
}

export interface ForgotPasswordRequest {
    email: string;
}

export interface ResetPasswordRequest {
    token: string;
    new_password: string;
}

export interface VerifyEmailRequestDto {
    email: string;
}

export interface VerifyEmailConfirmDto {
    token: string;
}
