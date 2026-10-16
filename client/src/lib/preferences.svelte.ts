import type { PreferencesPatch, PreferencesResponse } from "@/bindings/routes/users";
import { request } from "@/lib/utils";

let preferences: PreferencesResponse | null = $state(null);

export function get(): PreferencesResponse | null {
  return preferences;
}

// Fetches the caller's own preferences
export async function load(): Promise<void> {
  const response = await request("/me/preferences");
  preferences = await response.json() as PreferencesResponse;
}

// Writes any subset of the caller's own preferences
export async function save(patch: Partial<PreferencesPatch>): Promise<void> {
  await request("/me/preferences", "PATCH", patch);
}
