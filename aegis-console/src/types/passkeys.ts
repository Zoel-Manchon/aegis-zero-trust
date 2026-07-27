export interface PasskeyCredentialView {
    credential_id: string;
    friendly_name: string | null;
    transports: string[];
    created_at: string;
    last_used_at: string | null;
}

export interface PasskeyChallenge {
    challenge_id: string;
    public_key: unknown;
}
