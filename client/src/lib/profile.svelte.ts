import type { UserProfileResponse } from "@/bindings/routes/users";
import { getMe } from "@/lib/api/users";

let profile: UserProfileResponse | null = $state(null);

export function getProfile(): UserProfileResponse | null {
  return profile;
}

// Fetches the caller's own profile
export async function loadProfile(url: string, token: string): Promise<void> {
  profile = await getMe(url, token);
}
