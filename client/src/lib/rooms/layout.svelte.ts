import type { RoomListItem } from "@/bindings/shared";
import { listMyRooms } from "@/lib/api/rooms";
import { getPreferences, updatePreferences } from "@/lib/api/users";
import { setRoomData } from "@/lib/rooms/data.svelte";
import { getToken, getUrl } from "@/lib/session.svelte";

let layout: RoomListItem[] = $state([]);
let locked: boolean = $state(true);

export function getLayout(): RoomListItem[] {
  return layout;
}

export function setLayout(next: RoomListItem[]): void {
  layout = next;
}

// Moves a room or folder so it ends up at toIndex. A null folder means the top level
export function moveEntry(fromIndex: number, fromFolder: string | null, toIndex: number, toFolder: string | null): void {
  const source = listFor(fromFolder);
  const dest = listFor(toFolder);
  if (source === undefined || dest === undefined) return;

  const entry = source[fromIndex];
  if (entry === undefined) return;
  if (typeof entry !== "string" && toFolder !== null) return; // A folder can't sit inside a folder

  source.splice(fromIndex, 1);
  dest.splice(toIndex, 0, entry);
}

// The top level list, or one folder's rooms
function listFor(folder: string | null): RoomListItem[] | undefined {
  if (folder === null) {
    return layout;
  }

  for (const entry of layout) {
    if (typeof entry !== "string" && entry.name === folder) {
      return entry.rooms;
    }
  }
  return undefined;
}

// Removes a folder, its rooms take its place at the top level
export function deleteFolder(folderName: string): void {
  for (let index = 0; index < layout.length; index++) {
    const entry = layout[index];
    if (typeof entry !== "string" && entry.name === folderName) {
      layout.splice(index, 1, ...entry.rooms);
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

// Unlocks a locked layout, or locks and saves an unlocked one
export function toggleLock(): void {
  const url = getUrl();
  const token = getToken();
  if (url === null || token === null) return;

  if (locked) {
    unlockLayout();
  } else {
    lockLayout(url, token);
  }
}
