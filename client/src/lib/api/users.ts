import type { DeleteAccountRequest, ProfilePatch, SetRoleRequest, UserProfileResponse, UserSummaryResponse, UsersLookup } from "@/bindings/routes/users";
import { request } from "@/lib/utils";

export async function get(id: string): Promise<UserProfileResponse> {
    const response = await request(`/users/${id}`);
    return await response.json() as UserProfileResponse;
}

export async function lookup(body: UsersLookup): Promise<UserSummaryResponse[]> {
    const response = await request("/users/lookup", "POST", body);
    return await response.json() as UserSummaryResponse[];
}

export async function updateProfile(body: ProfilePatch): Promise<void> {
    await request("/me", "PATCH", body);
}

export async function changePassword(current_password: string, new_password: string, revoke_others: boolean): Promise<void> {
    await request("/me/password", "PATCH", { current_password, new_password, revoke_others });
}

export async function deleteMe(anonymize: boolean, delete_history: boolean): Promise<void> {
    await request("/me", "DELETE", { anonymize, delete_history });
}

export async function deleteUser(id: string, body: DeleteAccountRequest): Promise<void> {
    await request(`/users/${id}`, "DELETE", body);
}

export async function setRole(id: string, body: SetRoleRequest): Promise<void> {
    await request(`/users/${id}/role`, "PATCH", body);
}
