import type { PatchRoomPreferences } from "@/bindings/routes/me";
import type { UserPreferences } from "@/bindings/types/users";
import { request } from "@/lib/utils";

let preferences: UserPreferences | null = $state(null);

export function get(): UserPreferences | null {
  return preferences;
}

// Fetches the caller's own preferences
export async function load(): Promise<void> {
  const response = await request("/me/preferences");
  preferences = await response.json() as UserPreferences;
}

// Writes any subset of the caller's own preferences
export async function save(patch: Partial<PatchRoomPreferences>): Promise<void> {
  await request("/me/preferences", "PATCH", patch);
}
