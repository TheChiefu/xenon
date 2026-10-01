import type { Folder } from "@/bindings/shared";
import { getLayout, renameFolder, setLayout } from "@/lib/rooms/layout.svelte";

// The keydown event Svelte passes for a folder name field
type NameFieldEvent = KeyboardEvent & { currentTarget: HTMLInputElement };

// True while the inline "name this folder" field is showing
let creating: boolean = $state(false);

// The folder showing its rename field, null when none is
let renamingFolder: Folder | null = $state(null);

// Creating //
export function isCreatingFolder(): boolean {
  return creating;
}
export function startCreatingFolder(): void {
  creating = true;
}
export function stopCreatingFolder(): void {
  creating = false;
}

// Adds the folder on Enter, drops it on Escape or if left blank
export function finishCreatingFolder(event: NameFieldEvent): void {
  if (event.key === "Enter") {
    const name = event.currentTarget.value.trim();
    if (name) {
      addFolder(name);
    }
    creating = false;
    return;
  }

  if (event.key === "Escape") {
    creating = false;
  }
}

// Renaming //
export function isRenamingFolder(folder: Folder): boolean {
  return renamingFolder === folder;
}
export function startRenamingFolder(folder: Folder): void {
  renamingFolder = folder;
}
export function stopRenamingFolder(): void {
  renamingFolder = null;
}

// Applies the name on Enter, keeps the old one on Escape or if left blank
export function finishRenamingFolder(event: NameFieldEvent, folder: Folder): void {
  if (event.key === "Enter") {
    const name = event.currentTarget.value.trim();
    if (name) {
      renameFolder(folder, name);
    }
    renamingFolder = null;
    return;
  }

  if (event.key === "Escape") {
    renamingFolder = null;
  }
}

// Helper Methods //

// Adds a folder with nothing in it
function addFolder(name: string): void {
  setLayout([...getLayout(), { name, rooms: [], collapsed: false }]);
}
