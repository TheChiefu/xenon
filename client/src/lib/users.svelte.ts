// Summaries of users the client has looked up

import { SvelteMap } from "svelte/reactivity";
import type { UserSummaryResponse } from "@/bindings/routes/users";
import { lookupUsers } from "@/lib/api/users";
import { getServerInfo } from "@/lib/server.svelte";

const summaries: SvelteMap<string, UserSummaryResponse> = new SvelteMap();

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
