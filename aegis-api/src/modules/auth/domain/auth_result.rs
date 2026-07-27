pub enum RegisterResult {
    Success,
    EmailAlreadyExists,
    WeakPassword,
    InvalidCredentials,
}

pub enum LoginResult {
    Success {
        user_id: i64,
        access_token: String,
        refresh_token: String,
        jti: uuid::Uuid,
    },

    MfaRequired {
        user_id: i64,
        mfa_token: String,
    },

    InvalidCredentials,
}
