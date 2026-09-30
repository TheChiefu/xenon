import { getLayout, setLayout } from "@/lib/rooms/layout.svelte";

// Folder name to whether it's hiding its members
let collapsed: Record<string, boolean> = $state({});

// True while the inline "name this folder" field is showing
let creating: boolean = $state(false);

export function isCollapsed(name: string): boolean {
  return collapsed[name] === true;
}

export function toggleFolder(name: string): void {
  collapsed[name] = !collapsed[name];
}

export function isCreatingFolder(): boolean {
  return creating;
}

export function startCreatingFolder(): void {
  creating = true;
}

// Confirms the typed name, or drops the folder if left blank
export function confirmFolder(name: string): void {
  if (name.trim()) {
    addFolder(name.trim());
  }
  creating = false;
}

export function cancelFolder(): void {
  creating = false;
}

// Adds a folder with nothing in it yet
function addFolder(name: string): void {
  setLayout([...getLayout(), { name, rooms: [] }]);
}
