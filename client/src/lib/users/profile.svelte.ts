import type { UserProfile } from "@/bindings/types/users";
import * as users from "@/lib/users/data.svelte";
import { request } from "@/lib/utils";

let profile: UserProfile | null = $state(null);

export function get(): UserProfile | null {
  return profile;
}

// Fetches the caller's own profile
export async function load(): Promise<void> {
  const response = await request("/me");
  const fetched = await response.json() as UserProfile;
  profile = fetched;

  // Add own summary to user summaries
  users.cache({
    id: fetched.id,
    username: fetched.username,
    display_name: fetched.display_name,
    avatar_file_id: fetched.avatar_file_id,
    banner_file_id: fetched.banner_file_id,
    global_role: fetched.global_role,
  });
  users.loadProfileFiles(fetched.id);
}