import type { Folder, RoomListItem } from "@/bindings/shared";
import { listMyRooms } from "@/lib/api/rooms";
import { getPreferences, updatePreferences } from "@/lib/api/users";
import { setRoomData } from "@/lib/rooms/data.svelte";
import { getToken, getUrl } from "@/lib/session.svelte";

let layout: RoomListItem[] = $state([]);
let locked: boolean = $state(true);

// Layout Access //

export function getLayout(): RoomListItem[] {
  return layout;
}

export function setLayout(next: RoomListItem[]): void {
  layout = next;
}

// Layout Changes //

// Where a drop lands in relation to the row it was released on
export enum Drop {
  Above,
  Inside,
  Below,
}

// Puts an entry above, inside or below a target
export function moveEntry(moved: RoomListItem, target: RoomListItem, drop: Drop): void {
  if (moved === target) return;

  // A room dropped inside a folder becomes its first room
  if (drop === Drop.Inside) {
    if (typeof moved !== "string") return;
    if (typeof target === "string") return;

    const inside = listContaining(moved);
    inside.splice(inside.indexOf(moved), 1);
    target.rooms.unshift(moved);
    return;
  }

  const to = listContaining(target);

  // A folder can't go inside a folder
  if (typeof moved !== "string" && to !== layout) return;

  const from = listContaining(moved);
  from.splice(from.indexOf(moved), 1);

  let index = to.indexOf(target);
  if (drop === Drop.Below) {
    index += 1;
  }

  to.splice(index, 0, moved);
}

// Shows or hides the rooms inside a folder
export function toggleFolder(folder: Folder): void {
  folder.collapsed = !folder.collapsed;
  saveLayout();
}

export function renameFolder(folder: Folder, newName: string): void {
  folder.name = newName;
  saveLayout();
}

// Removes a folder, its rooms take its place
export function deleteFolder(folder: Folder): void {
  const index = layout.indexOf(folder);
  if (index === -1) return;

  layout.splice(index, 1, ...folder.rooms);
  saveLayout();
}

// Server //

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
      layout.push({ name: entry.name, rooms: roomIds, collapsed: entry.collapsed });
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
export async function saveLayout(): Promise<void> {
  const url = getUrl();
  const token = getToken();
  if (url === null || token === null) return;

  await updatePreferences(url, token, { room_layout: JSON.stringify(layout) });
}

// Lock //

export function isLocked(): boolean {
  return locked;
}

export function unlockLayout(): void {
  locked = false;
}

// Stops drag and drop, and saves the layout as it stands
export async function lockLayout(): Promise<void> {
  locked = true;
  await saveLayout();
}

// Unlocks a locked layout, or locks and saves an unlocked one
export function toggleLock(): void {
  if (locked) {
    unlockLayout();
  } else {
    lockLayout();
  }
}

// Helper Methods //

// The list an entry sits in, which is a folder's rooms or the layout
function listContaining(entry: RoomListItem): RoomListItem[] {
  if (typeof entry === "string") {
    for (const item of layout) {
      if (typeof item !== "string" && item.rooms.includes(entry)) return item.rooms;
    }
  }
  return layout;
}
