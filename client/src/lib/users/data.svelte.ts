// What the client stores for each user

import { SvelteMap } from "svelte/reactivity";
import type { ServerInfo } from "@/bindings/routes/server";
import type { UserSummaryResponse } from "@/bindings/routes/users";
import { Status } from "@/bindings/shared";
import { GamePresence, type Device, type GameActivity } from "@/bindings/sockets/events";
import * as usersAPI from "@/lib/api/users";
import { LocalEvent, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
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

let per_lookup: number = 1;

// SUMMARIES //

// What is held for a user, or undefined until a lookup fills it
export function get(id: string): UserSummaryResponse | undefined {
  return summaries.get(id);
}

// Fetches every id not already held
export async function load(ids: (string | null)[]): Promise<void> {

  // Find ids not already cached, skipping null
  const missing: string[] = [];
  for (const id of new Set(ids)) {
    if (id !== null && !summaries.has(id)) {
      missing.push(id);
    }
  }

  // Skip lookup when every user is cached
  if (missing.length === 0) {
    return;
  }

  for (let start = 0; start < missing.length; start += per_lookup) {
    const batch: string[] = missing.slice(start, start + per_lookup);
    const fetched = await usersAPI.lookup({ ids: batch });

    // Cache each fetched summary and sync its avatar and banner files
    for (const user of fetched) {
      summaries.set(user.id, user);
      loadProfileFiles(user.id);
    }
  }
}

// Replaces what is held, for a profile change the server reports
export function cache(user: UserSummaryResponse): void {
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

export function getDisplayName(id: string | null): string {

  if (id === null) {
    return "Deleted User";
  }

  const user = summaries.get(id);
  if (user === undefined) {
    return "Undefined User";
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

// EVENTS //

// Replace all user statuses from snapshot
events.subscribe(ServerEventType.presence_snapshot, (event) => {
  clearPresences();

  for (const user of event.users) {
    setPresence(user.user_id, user.status, user.device);
  }
});

// Update a user's status
events.subscribe(ServerEventType.presence_updated, (event) => {
  setPresence(event.user_id, event.status, event.device);
});

// Replace all game activities from snapshot
events.subscribe(ServerEventType.game_presence_snapshot, (event) => {
  clearActivities();

  for (const user of event.users) {
    setActivity(user.user_id, {
      platform: user.platform,
      status: user.status,
      title: user.title,
      detail: user.detail,
    });
  }
});

// Update a user's game activity
events.subscribe(ServerEventType.game_presence_updated, (event) => {
  setActivity(event.user_id, {
    platform: event.platform,
    status: event.status,
    title: event.title,
    detail: event.detail,
  });
});

// Update a stored user's name, avatar and banner
events.subscribe(ServerEventType.profile_updated, (event) => {
  const summary = summaries.get(event.user_id);
  if (summary === undefined) {
    return;
  }

  cache({
    id: event.user_id,
    username: summary.username,
    display_name: event.display_name,
    avatar_file_id: event.avatar_file_id,
    banner_file_id: event.banner_file_id,
    global_role: summary.global_role,
  });
  loadProfileFiles(event.user_id);
});

events.subscribe(LocalEvent.signed_out, clear); // Clear all users on sign out
events.subscribe(LocalEvent.server_info_loaded, (info: ServerInfo) => {
  per_lookup = info.limit_users_lookup;
});
