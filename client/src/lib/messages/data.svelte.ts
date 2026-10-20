// What the client stores for each room's messages

import { SvelteMap } from "svelte/reactivity";
import type { MessageResponse } from "@/bindings/routes/messages";
import * as messagesAPI from "@/lib/api/messages";
import { LocalEvent, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";

// Messages per room ID, each room's messages per message ID in the order they were added
const rooms: SvelteMap<string, SvelteMap<string, MessageResponse>> = new SvelteMap();

const olderMessagesRemain: SvelteMap<string, boolean> = new SvelteMap();

// LOADING //

export function hasOlderMessages(roomId: string): boolean {
  return olderMessagesRemain.get(roomId) ?? false;
}

// A room's stored messages
export function get(roomId: string): MessageResponse[] {
  const messages = rooms.get(roomId);
  if (messages === undefined) {
    return [];
  }

  return [...messages.values()];
}

// Fetches a room's newest batch, unless the room is already stored
export async function loadLatest(roomId: string): Promise<void> {
  if (rooms.has(roomId)) {
    return;
  }

  const fetched = await messagesAPI.fetch(roomId);
  rooms.set(roomId, toMap(fetched));
  olderMessagesRemain.set(roomId, fetched.length === messagesAPI.pageSize());
}

export async function loadOlder(roomId: string): Promise<void> {
  const oldest = get(roomId)[0];
  if (oldest === undefined) {
    return;
  }

  const fetched = await messagesAPI.fetch(roomId, oldest.seq);

  // Read after the request so messages that arrived meanwhile are kept
  rooms.set(roomId, toMap([...fetched, ...get(roomId)]));
  olderMessagesRemain.set(roomId, fetched.length === messagesAPI.pageSize());
}

// EVENTS //

// Adds a message to its stored room, unless its ID is already present
export function add(message: MessageResponse): void {
  const messages = rooms.get(message.room_id);
  if (messages === undefined || messages.has(message.id)) {
    return;
  }

  messages.set(message.id, message);
}

// Replaces a message's body and edit time
export function edit(roomId: string, messageId: string, body: string | null, editedAt: number): void {
  const messages = rooms.get(roomId);
  if (messages === undefined) {
    return;
  }

  const message = messages.get(messageId);
  if (message === undefined) {
    return;
  }

  messages.set(messageId, { ...message, body, edited_at: editedAt });
}

// Removes a message from its room
export function remove(roomId: string, messageId: string): void {
  const messages = rooms.get(roomId);
  if (messages === undefined) {
    return;
  }

  messages.delete(messageId);
}

// SESSION //

// Removes every stored room
export function clear(): void {
  rooms.clear();
  olderMessagesRemain.clear();
}

// HELPERS //

function toMap(list: MessageResponse[]): SvelteMap<string, MessageResponse> {
  const messages: SvelteMap<string, MessageResponse> = new SvelteMap();
  for (const message of list) {
    messages.set(message.id, message);
  }

  return messages;
}

// EVENTS //

// Store a new message
events.subscribe(ServerEventType.message, (event) => {
  add(event.message);
});

// Update an edited message
events.subscribe(ServerEventType.message_edited, (event) => {
  edit(event.room_id, event.message_id, event.body, event.edited_at);
});

// Remove a deleted message
events.subscribe(ServerEventType.message_deleted, (event) => {
  remove(event.room_id, event.message_id);
});

// Clear all messages on resync and sign out
events.subscribe(ServerEventType.resync, clear);
events.subscribe(LocalEvent.signed_out, clear);
