// How far the user has read in each room

import { SvelteMap } from "svelte/reactivity";
import * as roomsAPI from "@/lib/api/rooms";
import * as messages from "@/lib/messages/data.svelte";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

// Unread message count per room ID
const unread: SvelteMap<string, number> = new SvelteMap();

// Newest message seq the server has stored as read, per room ID
const positions: Map<string, number> = new Map();

export function getUnread(roomId: string): number {
  return unread.get(roomId) ?? 0;
}

export function addUnread(roomId: string): void {
  unread.set(roomId, getUnread(roomId) + 1);
}

// Fetches every room's read position and unread count
export async function load(): Promise<void> {
  const fetched = await roomsAPI.listMyReads();

  unread.clear();
  positions.clear();
  for (const entry of fetched) {
    unread.set(entry.room_id, entry.unread);
    positions.set(entry.room_id, entry.last_read_seq);
  }
}

// Stores the newest loaded message as read in a room being left
export function leave(roomId: string, keepalive: boolean): void {
  unread.set(roomId, 0);

  let newest = 0;
  for (const message of messages.get(roomId)) {
    newest = Math.max(newest, message.seq);
  }

  // Nothing new since the stored position
  if (newest <= (positions.get(roomId) ?? 0)) {
    return;
  }

  positions.set(roomId, newest);
  roomsAPI.markRead(roomId, newest, keepalive).catch(() => {});
}

// Removes every stored count and position
export function clear(): void {
  unread.clear();
  positions.clear();
}

events.subscribe(LocalEvent.signed_out, clear);
