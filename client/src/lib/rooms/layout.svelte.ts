import type { RoomResponse } from "@/bindings/routes/rooms";
import type { Folder, RoomListItem } from "@/bindings/shared";
import { listMyRooms } from "@/lib/api/rooms";
import { getStoredPreferences, savePreferences } from "@/lib/preferences.svelte";
import { addRoomData, setRoomData, type RoomId } from "@/lib/rooms/data.svelte";

// Note: any bare "string" types are assumed to be room rows

export enum Section {
  Above,
  Inside,
  Below,
}

let layout: RoomListItem[] = $state([]);  // Rows of room and folders
let locked: boolean = $state(true);       // Permission to change layout
let saved: string = $state("[]");         // Layout as JSON when last loaded or saved

// Getters & Setters //

export function getLayout(): RoomListItem[] {
  return layout;
}

// Layout Changes //

// Puts an entry above, inside or below a target
export function moveEntry(moved: RoomListItem, target: RoomListItem, section: Section): void {
  if (moved === target) return;

   // Target is room
  if (typeof target === "string") {
    place(moved, target, section);
    return;
  }

  // Target is folder
  if (typeof moved === "string") { // Moved is room

    // An open folder's bottom section counts as inside
    if (section === Section.Below && !target.collapsed) section = Section.Inside;
  }
  place(moved, target, section);
}

export function folderCreate(name: string): void {
  layout.push({ name, rooms: [], collapsed: false });
}

export function folderToggle(folder: Folder): void {
  folder.collapsed = !folder.collapsed;
}

export function folderRename(folder: Folder, newName: string): void {
  folder.name = newName;
}

export function folderDelete(folder: Folder): void {
  const index = layout.indexOf(folder);
  if (index === -1) return; // If folder is somehow not on layout, quick exit

  // Move folder's rooms back into layout
  layout.splice(index, 1, ...folder.rooms);
}

export function roomAdd(room: RoomResponse): void {
  addRoomData(room);
  layout.push(room.id);
}

// Server //

// Load room list for user
export async function loadRooms(): Promise<void> {
  const fetched = await listMyRooms(); // Fetch all rooms user has access to
  const stored = getStoredPreferences()?.room_layout; // User's saved arrangement
  const storedLayout = stored ? JSON.parse(stored) as RoomListItem[] : [];

  setRoomData(fetched);

  const seenIds: Set<RoomId> = new Set();
  layout = [];

  const fetchedIds: Set<RoomId> = new Set();
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
      const roomIds: RoomId[] = [];
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

  saved = JSON.stringify(layout);
}

// Persists the current layout
export async function saveLayout(): Promise<void> {
  const current = JSON.stringify(layout);
  await savePreferences({ room_layout: current });
  saved = current;
}

// True when the layout has changes that haven't been saved
export function isUnsaved(): boolean {
  return JSON.stringify(layout) !== saved;
}

// Lock //
export function isLocked(): boolean {
  return locked;
}
// Unlocks a locked layout, or locks an unlocked one
export function toggleLock(): void {
  locked = !locked;
}

// Helper Methods //

// Puts an entry at the top of a folder when inside, otherwise above or below the target
function place(moved: RoomListItem, target: RoomListItem, section: Section): void {

  // Dropped inside the target
  if (section === Section.Inside) {

    // Only rooms go inside folders
    if (typeof moved !== "string") return;
    if (typeof target === "string") return;

    // Reorder list moved came from (filling void)
    const from = listContaining(moved);
    from.splice(from.indexOf(moved), 1);

    // Insert moved at the top of the folder
    target.rooms.unshift(moved);
    return;
  }

  // Above or below the target, in the list the target sits in
  const to = listContaining(target);

  // A folder can't go inside a folder
  if (typeof moved !== "string" && to !== layout) return;

  // Reorder list moved came from (filling void)
  const from = listContaining(moved);
  from.splice(from.indexOf(moved), 1);

  // Insert moved above or below the target
  const index = to.indexOf(target) + (section === Section.Below ? 1 : 0);
  to.splice(index, 0, moved);
}

// The list an entry sits in, which is a folder's rooms or the layout
function listContaining(entry: RoomListItem): RoomListItem[] {
  if (typeof entry === "string") {
    for (const item of layout) {
      if (typeof item !== "string" && item.rooms.includes(entry)) return item.rooms;
    }
  }
  return layout;
}
