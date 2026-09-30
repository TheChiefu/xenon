import { getLayout, setLayout } from "@/lib/rooms/state.svelte";

// Names of folders currently hiding their members
let collapsed: Set<string> = $state(new Set());

// True while the inline "name this folder" field is showing
let creating: boolean = $state(false);

export function isCollapsed(name: string): boolean {
  return collapsed.has(name);
}

export function toggleFolder(name: string): void {
  if (collapsed.has(name)) {
    collapsed.delete(name);
  } else {
    collapsed.add(name);
  }
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
