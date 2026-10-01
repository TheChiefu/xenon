import type { Folder } from "@/bindings/shared";
import { folderRename, getLayout, setLayout } from "@/lib/rooms/layout.svelte";

// Types //
type NameFieldEvent = KeyboardEvent & { currentTarget: HTMLInputElement }; // Keyboard event from an <input>
enum Kind {
  Create, // Naming a new folder
  Rename, // Renaming an existing folder
}
type Naming = // Which folder name field is showing
  | { kind: Kind.Create }
  | { kind: Kind.Rename, folder: Folder }
  | null;

// State //
let naming: Naming = $state(null);

// Naming //
export function isCreating(): boolean {
  return naming?.kind === Kind.Create;
}
export function isRenaming(folder: Folder): boolean {
  return naming?.kind === Kind.Rename && naming.folder === folder;
}
export function startCreation(): void {
  naming = { kind: Kind.Create };
}
export function startRename(folder: Folder): void {
  naming = { kind: Kind.Rename, folder };
}

// Keyboard/Browser Events //
export function nameFinished(event: NameFieldEvent): void {

  // Applies the name on Enter
  if (event.key === "Enter") {
    const name = event.currentTarget.value.trim();
    
    // Empty names get dropped
    if (!name) {
      nameReset();
      return;
    }

    if (naming?.kind === Kind.Create) {
      // Add a new folder based on name (with nothing in it)
      setLayout([...getLayout(), { name, rooms: [], collapsed: false }]);
    } else if (naming?.kind === Kind.Rename) {
      // Rename selected folder with new name
      folderRename(naming.folder, name);
    }
    return;
  }

  // Reset naming state on Escape
  if (event.key === "Escape") {
    nameReset();
  }
}

export function nameReset(): void {
  naming = null;
}
