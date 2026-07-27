/* WebAuthn browser ceremonies.
 *
 * The base64url <-> ArrayBuffer marshalling that navigator.credentials requires
 * is delegated to @github/webauthn-json. The server speaks webauthn-rs, whose
 * begin responses are standard WebAuthn JSON ({ publicKey: ... }) and whose
 * finish endpoints accept the credential produced by `.toJSON()`. */

import {
    create,
    get,
    parseCreationOptionsFromJSON,
    parseRequestOptionsFromJSON,
    supported,
    type CredentialCreationOptionsJSON,
    type CredentialRequestOptionsJSON,
} from "@github/webauthn-json/browser-ponyfill";
import { passkeysApi } from "@/lib/api/passkeys";
import type { LoginResponse } from "@/types";

/** True when this browser exposes the WebAuthn API. */
export function passkeysSupported(): boolean {
    return supported();
}

/** Registration ceremony: begin -> navigator.credentials.create -> finish. */
export async function enrollPasskey(friendlyName?: string): Promise<void> {
    const challenge = await passkeysApi.registerBegin(friendlyName);
    const options = parseCreationOptionsFromJSON(
        challenge.public_key as CredentialCreationOptionsJSON,
    );
    const credential = await create(options);
    await passkeysApi.registerFinish({
        challenge_id: challenge.challenge_id,
        credential: credential.toJSON(),
        friendly_name: friendlyName,
    });
}

/** Authentication ceremony: begin -> navigator.credentials.get -> finish. */
export async function assertPasskey(email: string): Promise<LoginResponse> {
    const challenge = await passkeysApi.loginBegin(email);
    const options = parseRequestOptionsFromJSON(
        challenge.public_key as CredentialRequestOptionsJSON,
    );
    const credential = await get(options);
    return passkeysApi.loginFinish({
        challenge_id: challenge.challenge_id,
        credential: credential.toJSON(),
    });
}
