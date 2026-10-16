import type { ProfilePatch, UserSummaryResponse, UsersLookup } from "@/bindings/routes/users";
import { request } from "@/lib/utils";

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
