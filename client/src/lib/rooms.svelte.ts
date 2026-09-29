import type { MyRoomResponse } from "@/bindings/routes/rooms";
import type { RoomListItem } from "@/bindings/shared";
import { listMyRooms } from "@/lib/api/rooms";
import { getPreferences, updatePreferences } from "@/lib/api/users";

let layout: RoomListItem[] = $state([]);
let rooms: Map<string, MyRoomResponse> = $state(new Map());

export function getLayout(): RoomListItem[] {
  return layout;
}

export function getRoomData(id: string): MyRoomResponse | undefined {
  return rooms.get(id);
}

export function setLayout(next: RoomListItem[]): void {
  layout = next;
}


// Load room list for user
export async function loadRooms(url: string, token: string): Promise<void> {
  const fetched = await listMyRooms(url, token); // Fetch all rooms user has access to
  const preferences = await getPreferences(url, token); // Get user saved room layout preference
  const storedLayout = JSON.parse(preferences.room_layout) as RoomListItem[];

  // Fetched room ids lookup set
  const fetchedIds: Set<string> = new Set();
  for (const room of fetched) {
    fetchedIds.add(room.id);
  }

  const seenIds: Set<string> = new Set();
  layout = [];

  // Drop stored entry whose room no longer exists
  for (const entry of storedLayout) {
    if (fetchedIds.has(entry.id)) {
      layout.push(entry);
      seenIds.add(entry.id);
    }
  }

  // Unseen IDs are new, added to end of layout
  for (const room of fetched) {
    if (!seenIds.has(room.id)) {
      layout.push({ id: room.id, folder: null });
    }
  }
}

// Persists the current layout. Called only on an explicit save or app close,
// never on every move
export async function saveLayout(url: string, token: string): Promise<void> {
  await updatePreferences(url, token, { room_layout: JSON.stringify(layout) });
}
