import type { MyRoomResponse } from "@/bindings/routes/rooms";

let rooms: Map<string, MyRoomResponse> = $state(new Map());

export function getRoomData(id: string): MyRoomResponse | undefined {
  return rooms.get(id);
}

export function setRoomData(fetched: MyRoomResponse[]): void {
  rooms = new Map();
  for (const room of fetched) {
    rooms.set(room.id, room);
  }
}
