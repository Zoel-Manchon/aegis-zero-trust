export interface MfaSetupResponse {
    secret: string;
    otpauth_url: string;
}

export interface VerifyMfaRequest {
    code: string;
}
