/* Shared response envelope — mirrors core::response::api_response::ApiResponse.
 * Most endpoints return `{ data }` on success / `{ error }` on failure.
 * `/refresh` is the one exception (top-level tokens); the api client handles it. */

export interface ApiResponse<T> {
    data?: T;
    error?: ApiError;
}

export interface ApiError {
    code: string;
    message: string;
}

/* Severity ramp — matches the Rust audit SecuritySeverity, lowercased by serde. */
export type Severity = "info" | "low" | "medium" | "high" | "critical";

/* Role — mirrors auth::models::user_model::UserRole (serde lowercase). */
export type UserRole = "user" | "admin";
