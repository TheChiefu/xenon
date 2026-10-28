import type { RoomInfo } from "@/bindings/types/rooms";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

// A room's id, which is how rooms are stored in the layout
export type RoomId = string;

let rooms: Map<RoomId, RoomInfo> = $state(new Map());

export function get(id: RoomId): RoomInfo | undefined {
  return rooms.get(id);
}

export function list(): RoomInfo[] {
  return [...rooms.values()];
}

export function set(fetched: RoomInfo[]): void {
  rooms = new Map();
  for (const room of fetched) {
    rooms.set(room.id, room);
  }
  events.dispatch(LocalEvent.rooms_changed, [...rooms.keys()]);
}

export function add(room: RoomInfo): void {
  rooms.set(room.id, room);
  events.dispatch(LocalEvent.rooms_changed, [...rooms.keys()]);
}

// Removes every stored room
export function clear(): void {
  rooms = new Map();
  events.dispatch(LocalEvent.rooms_changed, []);
}

// Clear all rooms on sign out
events.subscribe(LocalEvent.signed_out, clear);
