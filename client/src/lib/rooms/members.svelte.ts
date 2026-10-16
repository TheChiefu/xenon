import { SvelteMap, SvelteSet } from "svelte/reactivity";
import * as roomsAPI from "@/lib/api/rooms";
import * as users from "@/lib/users.svelte";

const rooms: SvelteMap<string, SvelteSet<string>> = new SvelteMap();

// LOADING //

export function get(roomId: string): string[] {
  const members = rooms.get(roomId);
  if (members === undefined) {
    return [];
  }

  return [...members];
}

export function has(roomId: string, userId: string): boolean {
  return rooms.get(roomId)?.has(userId) ?? false;
}

export async function load(roomId: string): Promise<void> {
  if (rooms.has(roomId)) {
    return;
  }

  await loadRoom(roomId);
}

// EVENTS //

export function add(roomId: string, userId: string): void {
  const members = rooms.get(roomId);
  if (members === undefined) {
    return;
  }

  members.add(userId);
  users.load([userId]);
}

export function remove(roomId: string, userId: string): void {
  rooms.get(roomId)?.delete(userId);
}

export function leave(roomId: string): void {
  rooms.delete(roomId);
}

// SESSION //

export function clear(): void {
  rooms.clear();
}

// HELPERS //

// Fetches a room's member IDs and their summaries
async function loadRoom(roomId: string): Promise<void> {
  const entries = await roomsAPI.listMembers(roomId);
  const ids: string[] = entries.map((entry) => entry.user_id);

  // Replaces the room's stored member IDs
  rooms.set(roomId, new SvelteSet(ids));

  await users.load(ids);
}
