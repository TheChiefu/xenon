import type { RoomListItem } from "@/bindings/shared";
import { listMyRooms } from "@/lib/api/rooms";
import { getPreferences, updatePreferences } from "@/lib/api/users";
import { setRoomData } from "@/lib/rooms.svelte";

let layout: RoomListItem[] = $state([]);
let locked: boolean = $state(true);

export function getLayout(): RoomListItem[] {
  return layout;
}

export function setLayout(next: RoomListItem[]): void {
  layout = next;
}

// Reorders top-level entries, rooms and folders alike
export function reorderRoom(fromIndex: number, toIndex: number): void {
  const moved = layout.splice(fromIndex, 1);
  layout.splice(toIndex, 0, ...moved);
}

// Adds a room as a new folder member
export function addRoomToFolder(roomIndex: number, folderName: string): void {
  const entry = layout[roomIndex];
  if (typeof entry !== "string") return; // A folder can't sit inside a folder

  for (const other of layout) {
    if (typeof other !== "string" && other.name === folderName) {
      layout.splice(roomIndex, 1);
      other.rooms.push(entry);
      return;
    }
  }
}

// Load room list for user
export async function loadRooms(url: string, token: string): Promise<void> {
  const fetched = await listMyRooms(url, token); // Fetch all rooms user has access to
  const preferences = await getPreferences(url, token); // Get user saved room layout preference
  const storedLayout = JSON.parse(preferences.room_layout) as RoomListItem[];

  setRoomData(fetched);

  const seenIds: Set<string> = new Set();
  layout = [];

  const fetchedIds: Set<string> = new Set();
  for (const room of fetched) {
    fetchedIds.add(room.id);
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

// Persists the current layout
export async function saveLayout(url: string, token: string): Promise<void> {
  await updatePreferences(url, token, { room_layout: JSON.stringify(layout) });
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
