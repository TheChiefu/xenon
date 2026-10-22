import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { ActiveCall } from "@/bindings/sockets/events";
import { LocalEvent, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";

// Participants of each active call in the user's rooms, keyed by room id
const calls: SvelteMap<string, SvelteSet<string>> = new SvelteMap();

// GETTERS //

// Participants of a room's call
export function get(room_id: string): string[] {
  const participants = calls.get(room_id);

  if (participants === undefined) {
    return [];
  }

  return [...participants];
}

// LISTENERS //

// Replace every call with the server's list
events.subscribe(ServerEventType.call_snapshot, (event) => {
  calls.clear();

  const active: ActiveCall[] = event.calls;

  for (const call of active) {
    calls.set(call.room_id, new SvelteSet(call.participants));
  }
});

// Add a participant, starting the call if the room had none
events.subscribe(ServerEventType.participant_joined, (event) => {
  let participants = calls.get(event.room_id);

  if (participants === undefined) {
    participants = new SvelteSet();
    calls.set(event.room_id, participants);
  }

  participants.add(event.user_id);
});

// Remove a participant, ending the call once it is empty
events.subscribe(ServerEventType.participant_left, (event) => {
  const participants = calls.get(event.room_id);

  if (participants === undefined) {
    return;
  }

  participants.delete(event.user_id);

  if (participants.size === 0) {
    calls.delete(event.room_id);
  }
});

// Remove the call of a deleted room
events.subscribe(ServerEventType.room_deleted, (event) => {
  calls.delete(event.room_id);
});

// Clear all calls on sign out
events.subscribe(LocalEvent.signed_out, () => {
  calls.clear();
});
