import type { UserProfileResponse } from "@/bindings/routes/users";
import { setUser } from "@/lib/users.svelte";
import { request } from "@/lib/utils";

let profile: UserProfileResponse | null = $state(null);

export function getProfile(): UserProfileResponse | null {
  return profile;
}

// Fetches the caller's own profile
export async function loadProfile(): Promise<void> {
  const response = await request("/me");
  const fetched = await response.json() as UserProfileResponse;
  profile = fetched;

  // Add own summary to user summaries
  setUser({
    id: fetched.id,
    username: fetched.username,
    display_name: fetched.display_name,
    avatar_file_id: fetched.avatar_file_id,
    banner_file_id: fetched.banner_file_id,
  });
}