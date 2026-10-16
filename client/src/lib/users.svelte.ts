// What the client stores for each user

import { SvelteMap } from "svelte/reactivity";
import type { UserSummaryResponse } from "@/bindings/routes/users";
import { Status } from "@/bindings/shared";
import { GamePresence, type Device, type GameActivity } from "@/bindings/sockets/events";
import * as usersAPI from "@/lib/api/users";
import * as server from "@/lib/server.svelte";
import { fetchFileBlob, type FetchedFile } from "@/lib/utils";

// A user's status and what they are connected from
export interface Presence {
  status: Status;
  device: Device | null;
}

// Maps containing a user ID per type (summary, their status/presence, game activity)
const summaries: SvelteMap<string, UserSummaryResponse> = new SvelteMap();
const presences: SvelteMap<string, Presence> = new SvelteMap();
const activities: SvelteMap<string, GameActivity> = new SvelteMap();
const avatars: SvelteMap<string, FetchedFile> = new SvelteMap();
const banners: SvelteMap<string, FetchedFile> = new SvelteMap();

// SUMMARIES //

// What is held for a user, or undefined until a lookup fills it
export function get(id: string): UserSummaryResponse | undefined {
  return summaries.get(id);
}

// Fetches every id not already held
export async function load(ids: string[]): Promise<void> {
  const info = server.getInfo();
  if (info === null) {
    return;
  }

  const perLookup: number = info.limit_users_lookup;
  const missing: string[] = [...new Set(ids)].filter((id) => !summaries.has(id));

  for (let start = 0; start < missing.length; start += perLookup) {
    const fetched = await usersAPI.lookup({ ids: missing.slice(start, start + perLookup) });

    for (const user of fetched) {
      summaries.set(user.id, user);
    }
  }
}

// Replaces what is held, for a profile change the server reports
export function set(user: UserSummaryResponse): void {
  summaries.set(user.id, user);
}

// PRESENCE //

export function getPresence(id: string): Presence {
  return presences.get(id) ?? { status: Status.offline, device: null };
}

export function setPresence(id: string, status: Status, device: Device | null): void {
  presences.set(id, { status, device });
}

export function clearPresences(): void {
  presences.clear();
}

// GAMES //

export function getActivity(id: string): GameActivity | undefined {
  const gameActivity = activities.get(id);

  if (gameActivity?.status === GamePresence.Offline) {
    return undefined;
  }

  return gameActivity;
}

export function setActivity(id: string, game: GameActivity): void {
  activities.set(id, game);
}

export function clearActivities(): void {
  activities.clear();
}

// PROFILE FILES //

export function getDisplayName(id: string): string {
  const user = summaries.get(id);
  if (user === undefined) {
    return "";
  }

  return user.display_name;
}

export function getAvatar(id: string): FetchedFile | undefined {
  return avatars.get(id);
}

export function getBanner(id: string): FetchedFile | undefined {
  return banners.get(id);
}

// Downloads a user's changed avatar and banner
export function loadProfileFiles(id: string): void {
  const user = summaries.get(id);
  if (user === undefined) {
    return;
  }

  syncFile(avatars, id, user.avatar_file_id);
  syncFile(banners, id, user.banner_file_id);
}

// Replaces a user's entry in a file map when the id changed
// - files: Files keyed by user id
// - user_id: Id of a user
// - file_id: Id of a file to download
async function syncFile(
  files: SvelteMap<string, FetchedFile>,
  user_id: string,
  file_id: string | null,
): Promise<void> {
  const stored_file = files.get(user_id);

  // Read the id of a file already in the map to compare against
  let stored_file_id: string | null = null;
  if (stored_file !== undefined && stored_file.id !== undefined) {
    stored_file_id = stored_file.id;
  }

  // Skip if the given file is already stored
  if (file_id === stored_file_id) {
    return;
  }

  // Remove file ID from file map if it is null
  // Otherwise add file ID to map, adding it
  if (file_id === null) {
    files.delete(user_id);
  } else {
    files.set(user_id, await fetchFileBlob(file_id));
  }

  // Revoke a replaced file's URL
  if (stored_file !== undefined) {
    URL.revokeObjectURL(stored_file.url);
  }
}

// SESSION //

// Removes every stored user, revoking the URLs of their avatars and banners
export function clear(): void {
  for (const avatar of avatars.values()) {
    URL.revokeObjectURL(avatar.url);
  }

  for (const banner of banners.values()) {
    URL.revokeObjectURL(banner.url);
  }

  summaries.clear();
  presences.clear();
  activities.clear();
  avatars.clear();
  banners.clear();
}
