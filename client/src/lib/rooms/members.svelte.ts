import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { Notify, Permission } from "@/bindings/shared";
import * as roomsAPI from "@/lib/api/rooms";
import { LocalEvent, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
import * as users from "@/lib/users/data.svelte";

// Each member and what room they belong to
const rooms: SvelteMap<string, SvelteSet<string>> = new SvelteMap();

// Each member's permissions, keyed by room ID and then user ID
const permissions: SvelteMap<string, SvelteMap<string, Permission[]>> = new SvelteMap();

const notifies: SvelteMap<string, Notify> = new SvelteMap();

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

export function getPermissions(roomId: string, userId: string): Permission[] {
  return permissions.get(roomId)?.get(userId) ?? [];
}

export function hasPermissions(roomId: string, userId: string): boolean {
  return permissions.get(roomId)?.has(userId) ?? false;
}

export function getNotify(roomId: string): Notify | undefined {
  return notifies.get(roomId);
}

export function setNotify(roomId: string, notify: Notify): void {
  notifies.set(roomId, notify);
}

export function setPermissions(roomId: string, userId: string, granted: Permission[]): void {
  roomPermissions(roomId).set(userId, granted);
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
  permissions.get(roomId)?.delete(userId);
}

export function leave(roomId: string): void {
  rooms.delete(roomId);
  permissions.delete(roomId);
  notifies.delete(roomId);
}

// SESSION //

export function clear(): void {
  rooms.clear();
  permissions.clear();
  notifies.clear();
}

// HELPERS //

// Fetches a room's member IDs and their summaries
async function loadRoom(roomId: string): Promise<void> {
  const entries = await roomsAPI.listMembers(roomId);
  const ids: string[] = entries.map((entry) => entry.user_id);

  // Get users in room
  await users.load(ids);

  // Replaces the room's stored member IDs
  rooms.set(roomId, new SvelteSet(ids));

  for (const entry of entries) {
    setPermissions(roomId, entry.user_id, entry.permissions);

    if (entry.notify !== null) {
      notifies.set(roomId, entry.notify);
    }
  }
}

// Gets a room's permission map, creating it when missing
function roomPermissions(roomId: string): SvelteMap<string, Permission[]> {
  let room = permissions.get(roomId);
  if (room === undefined) {
    room = new SvelteMap();
    permissions.set(roomId, room);
  }

  return room;
}

// LISTENERS //

// Add a member who joined
events.subscribe(ServerEventType.member_joined, (event) => {
  add(event.room_id, event.user_id);
});

// Remove a member who left
events.subscribe(ServerEventType.member_left, (event) => {
  remove(event.room_id, event.user_id);
});

// Remove the members of a deleted room
events.subscribe(ServerEventType.room_deleted, (event) => {
  leave(event.room_id);
});

// Clear all members on resync and sign out
events.subscribe(ServerEventType.resync, clear);
events.subscribe(LocalEvent.signed_out, clear);
