// Opening and leaving rooms

import * as session from "@/lib/connection/session.svelte";
import { ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
import * as read from "@/lib/rooms/read.svelte";
import * as profile from "@/lib/users/profile.svelte";

// Opens a room, storing the one being left as read
export function select(roomId: string | null): void {
  const left = session.getSelectedRoom();
  if (left !== null && left !== roomId) {
    read.leave(left, false);
  }

  session.selectRoom(roomId);
}

// EVENTS //

// Counts a message from someone else in a room that is not open
events.subscribe(ServerEventType.message, (event) => {
  if (event.room_id === session.getSelectedRoom()) {
    return;
  }

  if (event.message.author_id === profile.get()?.id) {
    return;
  }

  read.addUnread(event.room_id);
});

// Stores the open room as read when the page closes
window.addEventListener("pagehide", () => {
  const open = session.getSelectedRoom();
  if (open !== null) {
    read.leave(open, true);
  }
});
