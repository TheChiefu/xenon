import type { RoomResponse } from "@/bindings/routes/rooms";

// A room's id, which is how rooms are stored in the layout
export type RoomId = string;

let rooms: Map<RoomId, RoomResponse> = $state(new Map());

export function getRoomData(id: RoomId): RoomResponse | undefined {
  return rooms.get(id);
}

export function setRoomData(fetched: RoomResponse[]): void {
  rooms = new Map();
  for (const room of fetched) {
    rooms.set(room.id, room);
  }
}

export function addRoomData(room: RoomResponse): void {
  rooms.set(room.id, room);
}

// Room Selection
let selectedRoom: RoomId | null = $state(null);

export function getSelectedRoom(): RoomId | null {
  return selectedRoom;
}

export function selectRoom(id: RoomId): void {
  selectedRoom = id;
}
