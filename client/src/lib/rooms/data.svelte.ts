import type { RoomResponse } from "@/bindings/routes/rooms";

// A room's id, which is how rooms are stored in the layout
export type RoomId = string;

let rooms: Map<RoomId, RoomResponse> = $state(new Map());

export function get(id: RoomId): RoomResponse | undefined {
  return rooms.get(id);
}

export function set(fetched: RoomResponse[]): void {
  rooms = new Map();
  for (const room of fetched) {
    rooms.set(room.id, room);
  }
}

export function add(room: RoomResponse): void {
  rooms.set(room.id, room);
}

// Removes every stored room
export function clear(): void {
  rooms = new Map();
}
