import type { RequestDeleteAccount, PatchUserProfile } from "@/bindings/routes/me";
import type { RequestRoleSet, RespondSummary, RequestUsers } from "@/bindings/routes/users";
import type { UserProfile } from "@/bindings/types/users";
import { request } from "@/lib/utils";

export async function get(id: string): Promise<UserProfile> {
    const response = await request(`/users/${id}`);
    return await response.json() as UserProfile;
}

export async function lookup(body: RequestUsers): Promise<RespondSummary[]> {
    const response = await request("/users/lookup", "POST", body);
    return await response.json() as RespondSummary[];
}

export async function updateProfile(body: PatchUserProfile): Promise<void> {
    await request("/me", "PATCH", body);
}

export async function changePassword(current_password: string, new_password: string, revoke_others: boolean): Promise<void> {
    await request("/me/password", "PATCH", { current_password, new_password, revoke_others });
}

export async function deleteMe(delete_history: boolean): Promise<void> {
    await request("/me", "DELETE", { delete_history });
}

export async function deleteUser(id: string, body: RequestDeleteAccount): Promise<void> {
    await request(`/users/${id}`, "DELETE", body);
}

export async function setRole(id: string, body: RequestRoleSet): Promise<void> {
    await request(`/users/${id}/role`, "PATCH", body);
}

export async function ban(id: string): Promise<void> {
    await request(`/users/${id}/ban`, "PUT");
}

export async function unban(id: string): Promise<void> {
    await request(`/users/${id}/ban`, "DELETE");
}
