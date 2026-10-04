import type { Folder } from "@/bindings/shared";
import type { MenuItem } from "@/lib/menu.svelte";
import type { RoomId } from "@/lib/rooms/data.svelte";
import { folderDelete, isLocked, toggleLock } from "@/lib/rooms/layout.svelte";
import { startCreation, startRename } from "@/lib/rooms/folders.svelte";
import { open } from "@/lib/rooms/creation.svelte";

// Right click on empty space in room pane
export function paneCtx(): MenuItem[] {
  const items: MenuItem[] = [];

  if (isLocked()) {
    items.push({ label: "Unlock Layout", action: toggleLock });
  } else {
    items.push({ label: "Lock Layout", action: toggleLock });
  }

  items.push({ label: "Create Folder", action: startCreation });

  items.push({ label: "New Room", action: open });

  return items;
}

// Right click on a room
export function roomCtx(roomId: RoomId): MenuItem[] {
  const items: MenuItem[] = [];

  items.push({
    label: "Copy Room ID",
    action: () => navigator.clipboard.writeText(roomId)
  });
  return items;
}

// Right click on a folder
export function folderCtx(folder: Folder): MenuItem[] {
  const items: MenuItem[] = [];

  items.push({
    label: "Rename Folder",
    action: () => startRename(folder)
  });

  items.push({
    label: "Delete Folder",
    action: () => folderDelete(folder)
  });

  items.push({
    label: "Copy Folder Name",
    action: () => navigator.clipboard.writeText(folder.name)
  });
  return items;
}
