/* Attack-range API — admin-only. Drives the SOC "launch attack from origin"
 * control that mirrors the reference lab's red-team loop. */

import { api } from "@/lib/api/client";

export interface ScenarioInfo {
    key: string;
    label: string;
    description: string;
}

export interface OriginPreset {
    key: string;
    label: string;
    ip: string;
}

export interface ScenariosResponse {
    scenarios: ScenarioInfo[];
    origins: OriginPreset[];
}

export interface GeoPointLite {
    country: string;
    city: string;
    lat: number;
    lon: number;
}

export interface LaunchReport {
    scenario: string;
    origin_ip: string;
    origin: GeoPointLite;
    events_recorded: number;
    impossible_travel: boolean;
    distance_km: number;
    speed_kmh: number;
    from: GeoPointLite | null;
}

export interface LaunchRequest {
    scenario: string;
    origin: string;
    victim_email: string;
}

export const attackRangeApi = {
    scenarios: () => api.get<ScenariosResponse>("/attack-range/scenarios"),
    launch: (body: LaunchRequest) => api.post<LaunchReport>("/attack-range/launch", body),
};
