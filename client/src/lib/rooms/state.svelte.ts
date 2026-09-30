import type { MyRoomResponse } from "@/bindings/routes/rooms";
import type { RoomListItem } from "@/bindings/shared";
import { listMyRooms } from "@/lib/api/rooms";
import { getPreferences, updatePreferences } from "@/lib/api/users";

let layout: RoomListItem[] = $state([]);
let rooms: Map<string, MyRoomResponse> = $state(new Map());
let selectedRoom: string | null = $state(null);
let locked: boolean = $state(true);

export function getLayout(): RoomListItem[] {
  return layout;
}

export function getRoomData(id: string): MyRoomResponse | undefined {
  return rooms.get(id);
}

export function setLayout(next: RoomListItem[]): void {
  layout = next;
}

// Move the entry at one layout position to another, shifting the entries between
export function moveRoom(from: number, to: number): void {
  const moved = layout.splice(from, 1);
  layout.splice(to, 0, ...moved);
}

// Load room list for user
export async function loadRooms(url: string, token: string): Promise<void> {
  const fetched = await listMyRooms(url, token); // Fetch all rooms user has access to
  const preferences = await getPreferences(url, token); // Get user saved room layout preference
  const storedLayout = JSON.parse(preferences.room_layout) as RoomListItem[];

  // Reset properties
  const seenIds: Set<string> = new Set();
  layout = [];
  rooms = new Map();

  // Parse each fetched room
  const fetchedIds: Set<string> = new Set();
  for (const room of fetched) {
    fetchedIds.add(room.id);  // Store IDs in set for fast lookup in layout sorting
    rooms.set(room.id, room); // Update global room map with up to date info
  }

  // Drop any room id that's gone
  for (const entry of storedLayout) {
    if (typeof entry === "string") { // Room ID
      if (fetchedIds.has(entry)) {
        layout.push(entry);
        seenIds.add(entry);
      }
    } else { // Folder
      const roomIds: string[] = [];
      for (const id of entry.rooms) {
        if (fetchedIds.has(id)) {
          roomIds.push(id);
          seenIds.add(id);
        }
      }
      layout.push({ name: entry.name, rooms: roomIds });
    }
  }

  // Unseen IDs are new, added to end of layout
  for (const room of fetched) {
    if (!seenIds.has(room.id)) {
      layout.push(room.id);
    }
  }
}

// Persists the current layout. Called only on an explicit save or app close,
// never on every move
export async function saveLayout(url: string, token: string): Promise<void> {
  await updatePreferences(url, token, { room_layout: JSON.stringify(layout) });
}

export function getSelectedRoom(): string | null {
  return selectedRoom;
}

export function selectRoom(id: string): void {
  selectedRoom = id;
}

export function isLocked(): boolean {
  return locked;
}

// Stops drag and drop, and saves the layout as it stands
export async function lockLayout(url: string, token: string): Promise<void> {
  locked = true;
  await saveLayout(url, token);
}

export function unlockLayout(): void {
  locked = false;
}