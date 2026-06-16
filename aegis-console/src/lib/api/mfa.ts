/* MFA self-service (authenticated). Mirrors mfa::interface::routes. */

import { api } from "@/lib/api/client";
import type { MfaSetupResponse, VerifyMfaRequest } from "@/types";

export const mfaApi = {
    /** Begin enrollment — returns the TOTP secret + otpauth:// URI for a QR. */
    setup: () => api.post<MfaSetupResponse>("/mfa/setup"),

    /** Confirm enrollment with the first code from the authenticator app. */
    verifySetup: (body: VerifyMfaRequest) => api.post<string>("/mfa/verify-setup", body),

    /** Step-up verification within an active session. */
    verify: (body: VerifyMfaRequest) => api.post<string>("/mfa/verify", body),

    /** Disable MFA (requires a current code). */
    disable: (body: VerifyMfaRequest) => api.post<string>("/mfa/disable", body),
};
