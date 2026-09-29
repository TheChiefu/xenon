import type { UserProfileResponse } from "@/bindings/routes/users";
import type { PreferencesPatch, PreferencesResponse } from "@/bindings/routes/users";

export async function getMe(url: string, token: string): Promise<UserProfileResponse> {
    const response = await fetch(`${url}/me`, {
        headers: { "Authorization": `Bearer ${token}` },
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return data as UserProfileResponse;
}

export async function getPreferences(url: string, token: string): Promise<PreferencesResponse> {
    const response = await fetch(`${url}/me/preferences`, {
        headers: { "Authorization": `Bearer ${token}` },
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return data as PreferencesResponse;
}

export async function updatePreferences(url: string, token: string, patch: Partial<PreferencesPatch>): Promise<void> {
    const response = await fetch(`${url}/me/preferences`, {
        method: "PATCH",
        headers: {
            "Authorization": `Bearer ${token}`,
            "Content-Type": "application/json",
        },
        body: JSON.stringify(patch),
    });

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error);
    }
}
