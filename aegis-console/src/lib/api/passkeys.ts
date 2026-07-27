/* Passkeys / WebAuthn API client. Verification is performed server-side by
 * webauthn-rs; the browser ceremonies live in lib/auth/webauthn.ts. */

import { api } from "@/lib/api/client";
import type { LoginResponse, PasskeyChallenge, PasskeyCredentialView } from "@/types";

interface ListResponse {
    passkeys: PasskeyCredentialView[];
}

export const passkeysApi = {
    list: () => api.get<ListResponse>("/passkeys").then((r) => r?.passkeys ?? []),

    remove: (credential_id: string) =>
        api.delete<string>("/passkeys", { credential_id }),

    registerBegin: (friendly_name?: string) =>
        api.post<PasskeyChallenge>("/passkeys/register/begin", { friendly_name }),

    registerFinish: (body: Record<string, unknown>) =>
        api.post<string>("/passkeys/register/finish", body),

    loginBegin: (email: string) =>
        api.post<PasskeyChallenge>("/passkeys/login/begin", { email }, { auth: false }),

    loginFinish: (body: Record<string, unknown>) =>
        api.post<LoginResponse>("/passkeys/login/finish", body, { auth: false }),
};
