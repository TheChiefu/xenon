import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { RtcRoom } from "@/bindings/rtc/events";
import { LocalEvent, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";

// Users connected over RTC in each of the user's rooms, keyed by room id
const rooms: SvelteMap<string, SvelteSet<string>> = new SvelteMap();

// GETTERS //

export function get(room_id: string): string[] {
  const participants = rooms.get(room_id);

  if (participants === undefined) {
    return [];
  }

  return [...participants];
}

// LISTENERS //

events.subscribe(ServerEventType.rtc_snapshot, (event) => {
  rooms.clear();

  const snapshot: RtcRoom[] = event.rooms;

  for (const room of snapshot) {
    rooms.set(room.room_id, new SvelteSet(room.participants));
  }
});

events.subscribe(ServerEventType.rtc_joined, (event) => {
  let participants = rooms.get(event.room_id);

  if (participants === undefined) {
    participants = new SvelteSet();
    rooms.set(event.room_id, participants);
  }

  participants.add(event.user_id);
});

events.subscribe(ServerEventType.rtc_left, (event) => {
  const participants = rooms.get(event.room_id);

  if (participants === undefined) {
    return;
  }

  participants.delete(event.user_id);

  if (participants.size === 0) {
    rooms.delete(event.room_id);
  }
});

events.subscribe(ServerEventType.room_deleted, (event) => {
  rooms.delete(event.room_id);
});

events.subscribe(LocalEvent.signed_out, () => {
  rooms.clear();
});
