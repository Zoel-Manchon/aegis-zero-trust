/* Passkeys / WebAuthn. The backend WebAuthn verification is still stubbed, so
 * list + delete are fully functional while register/login are experimental. */

import { api } from "@/lib/api/client";
import type { PasskeyChallenge, PasskeyCredentialView } from "@/types";

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
        api.post<unknown>("/passkeys/login/finish", body, { auth: false }),
};
