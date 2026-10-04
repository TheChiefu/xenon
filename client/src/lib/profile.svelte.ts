import type { UserProfileResponse } from "@/bindings/routes/users";
import { request } from "@/lib/utils";

let profile: UserProfileResponse | null = $state(null);

export function getProfile(): UserProfileResponse | null {
  return profile;
}

// Fetches the caller's own profile
export async function loadProfile(): Promise<void> {
  const response = await request("/me");
  profile = await response.json() as UserProfileResponse;
}
