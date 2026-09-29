import type { PreferencesResponse } from "@/bindings/routes/users";
import type { Status } from "@/bindings/shared";
import { getPreferences, updatePreferences } from "@/lib/api/users";

let preferences: PreferencesResponse | null = $state(null);

export function getStoredPreferences(): PreferencesResponse | null {
  return preferences;
}

// Fetches the caller's own preferences
export async function loadPreferences(url: string, token: string): Promise<void> {
  preferences = await getPreferences(url, token);
}

// Writes the caller's status to the server
export async function setStatus(url: string, token: string, status: Status): Promise<void> {
  await updatePreferences(url, token, { status });
  if (preferences) preferences.status = status;
}
