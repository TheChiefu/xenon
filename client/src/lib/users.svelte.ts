// What the client stores for each user

import { SvelteMap } from "svelte/reactivity";
import type { UserSummaryResponse } from "@/bindings/routes/users";
import { Status } from "@/bindings/shared";
import { GamePresence, type Device, type GameActivity } from "@/bindings/sockets/events";
import { lookupUsers } from "@/lib/api/users";
import { getServerInfo } from "@/lib/server.svelte";

// A user's status and what they are connected from
export interface Presence {
  status: Status;
  device: Device | null;
}

// Maps containing a user ID per type (summary, their status/presence, game activity)
const summaries: SvelteMap<string, UserSummaryResponse> = new SvelteMap();
const presences: SvelteMap<string, Presence> = new SvelteMap();
const activities: SvelteMap<string, GameActivity> = new SvelteMap();

// SUMMARIES //

// What is held for a user, or undefined until a lookup fills it
export function getUser(id: string): UserSummaryResponse | undefined {
  return summaries.get(id);
}

// Fetches every id not already held
export async function loadUsers(ids: string[]): Promise<void> {
  const info = getServerInfo();
  if (info === null) {
    return;
  }

  const perLookup: number = info.limit_users_lookup;
  const missing: string[] = [...new Set(ids)].filter((id) => !summaries.has(id));

  for (let start = 0; start < missing.length; start += perLookup) {
    const fetched = await lookupUsers({ ids: missing.slice(start, start + perLookup) });

    for (const user of fetched) {
      summaries.set(user.id, user);
    }
  }
}

// Replaces what is held, for a profile change the server reports
export function setUser(user: UserSummaryResponse): void {
  summaries.set(user.id, user);
}

// PRESENCE //

export function getPresenceOfUser(id: string): Presence {
  return presences.get(id) ?? { status: Status.offline, device: null };
}

export function setPresenceForUser(id: string, status: Status, device: Device | null): void {
  presences.set(id, { status, device });
}

export function clearAllPresences(): void {
  presences.clear();
}

// GAMES //

export function getActivityOfUser(id: string): GameActivity | undefined {
  const gameActivity = activities.get(id);

  if (gameActivity?.status === GamePresence.Offline) {
    return undefined;
  }

  return gameActivity;
}

export function setActivityForUser(id: string, game: GameActivity): void {
  activities.set(id, game);
}

export function clearAllActivities(): void {
  activities.clear();
}
