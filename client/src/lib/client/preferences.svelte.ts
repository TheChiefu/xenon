import type { PatchRoomPreferences, RespondRoomPreferences } from "@/bindings/routes/me";
import { request } from "@/lib/utils";

let preferences: RespondRoomPreferences | null = $state(null);

export function get(): RespondRoomPreferences | null {
  return preferences;
}

// Fetches the caller's own preferences
export async function load(): Promise<void> {
  const response = await request("/me/preferences");
  preferences = await response.json() as RespondRoomPreferences;
}

// Writes any subset of the caller's own preferences
export async function save(patch: Partial<PatchRoomPreferences>): Promise<void> {
  await request("/me/preferences", "PATCH", patch);
}
