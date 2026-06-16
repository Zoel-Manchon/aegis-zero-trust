import type { Severity } from "@/types/common";

/* Mirrors admin::security::domain::security_event_view::SecurityEventView */
export interface SecurityEvent {
    id: string;
    user_id: number | null;
    event_type: string;
    severity: Severity;
    ip_address: string | null;
    user_agent: string | null;
    session_id: string | null;
    jti: string | null;
    family_id: string | null;
    metadata: Record<string, unknown>;
    created_at: string;
}

/* Mirrors admin::security::domain::security_metric::SecurityMetrics */
export interface SecurityMetrics {
    total_events: number;
    critical_events: number;
    high_events: number;
    refresh_replays: number;
    policy_denials: number;
    mfa_failures: number;
    brute_force_lockouts: number;
}

/* Mirrors admin::security::domain::security_alert::SecurityAlert */
export interface SecurityAlert {
    alert_type: string;
    severity: Severity;
    title: string;
    description: string;
    count: number;
    metadata?: Record<string, unknown>;
}
