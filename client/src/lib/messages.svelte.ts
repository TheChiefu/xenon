// What the client stores for each room's messages

import { SvelteMap } from "svelte/reactivity";
import type { MessageResponse } from "@/bindings/routes/messages";
import { fetchMessages } from "@/lib/api/messages";

// Messages per room ID, each room's messages per message ID in the order they were added
const rooms: SvelteMap<string, SvelteMap<string, MessageResponse>> = new SvelteMap();

// LOADING //

// A room's stored messages
export function getMessages(roomId: string): MessageResponse[] {
  const messages = rooms.get(roomId);
  if (messages === undefined) {
    return [];
  }

  return [...messages.values()];
}

// Fetches a room's newest page, unless the room is already stored
export async function loadLatest(roomId: string): Promise<void> {
  if (rooms.has(roomId)) {
    return;
  }

  await loadRoom(roomId);
}

// Replaces each stored room with its newest page
export async function reloadLoaded(): Promise<void> {
  const roomIds = [...rooms.keys()];

  for (const roomId of roomIds) {
    await loadRoom(roomId);
  }
}

// EVENTS //

// Adds a message to its stored room, unless its ID is already present
export function addMessage(message: MessageResponse): void {
  const messages = rooms.get(message.room_id);
  if (messages === undefined || messages.has(message.id)) {
    return;
  }

  messages.set(message.id, message);
}

// Replaces a message's body and edit time
export function setMessageEdited(roomId: string, messageId: string, body: string | null, editedAt: number): void {
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
export function removeMessage(roomId: string, messageId: string): void {
  const messages = rooms.get(roomId);
  if (messages === undefined) {
    return;
  }

  messages.delete(messageId);
}

// SESSION //

// Removes every stored room
export function clearMessages(): void {
  rooms.clear();
}

// HELPERS //

// Fetches a room's newest page and stores it in place of what the room had
async function loadRoom(roomId: string): Promise<void> {
  const fetched = await fetchMessages(roomId);

  const messages: SvelteMap<string, MessageResponse> = new SvelteMap();
  for (const message of fetched) {
    messages.set(message.id, message);
  }

  rooms.set(roomId, messages);
}
