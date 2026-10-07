import type { UserSummaryResponse, UsersLookup } from "@/bindings/routes/users";
import { request } from "@/lib/utils";

export async function lookupUsers(body: UsersLookup): Promise<UserSummaryResponse[]> {
    const response = await request("/users/lookup", "POST", body);
    return await response.json() as UserSummaryResponse[];
}
