/* Auth + identity endpoints. Mirrors auth::interface::http::routes. */

import { api } from "@/lib/api/client";
import type {
    ForgotPasswordRequest,
    LoginRequest,
    LoginResponse,
    MeResponse,
    MfaCompleteRequest,
    RefreshRequest,
    RefreshResponse,
    RegisterRequest,
    ResetPasswordRequest,
    VerifyEmailConfirmDto,
    VerifyEmailRequestDto,
} from "@/types";

export const authApi = {
    register: (body: RegisterRequest) => api.post<string>("/register", body, { auth: false }),

    login: (body: LoginRequest) => api.post<LoginResponse>("/login", body, { auth: false }),

    /** /refresh returns tokens at top level, not wrapped in { data }. */
    refresh: (body: RefreshRequest) =>
        api.post<RefreshResponse>("/refresh", body, { auth: false, rawResponse: true }),

    logout: () => api.post<unknown>("/logout"),
    logoutAll: () => api.post<unknown>("/logout-all"),

    /** Exchange an mfa_token + TOTP code for a full token set. Public. */
    completeMfa: (body: MfaCompleteRequest) =>
        api.post<LoginResponse>("/mfa/complete-login", body, { auth: false }),

    me: () => api.get<MeResponse>("/me"),
};

export const passwordApi = {
    forgot: (body: ForgotPasswordRequest) =>
        api.post<unknown>("/password/forgot", body, { auth: false }),
    reset: (body: ResetPasswordRequest) =>
        api.post<unknown>("/password/reset", body, { auth: false }),
};

export const verifyEmailApi = {
    request: (body: VerifyEmailRequestDto) =>
        api.post<unknown>("/verify-email/request", body, { auth: false }),
    confirm: (body: VerifyEmailConfirmDto) =>
        api.post<unknown>("/verify-email/confirm", body, { auth: false }),
};
