import type { UserProfileResponse } from "@/bindings/routes/users";

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
