import { getLayout, isLocked, moveEntry } from "@/lib/rooms/layout.svelte";

// What a row shows while something is dragged over it
export enum DropMark {
  Above, // Line on the top edge
  Below, // Line on the bottom edge
  Into,  // Outline, the held room goes into this folder
}

// Position of the held row, null when nothing is held
let heldIndex: number | null = $state(null);

// Folder the held row is in, null for a top level row
let heldFolder: string | null = $state(null);

// Position of the row under the pointer, null when releasing changes nothing
let hoveredIndex: number | null = $state(null);

// Folder the hovered row is in, null for a top level row
let hoveredFolder: string | null = $state(null);

// True when the pointer is in the bottom half of the hovered row
let hoveredBelow: boolean = $state(false);

export function isDragging(): boolean {
  return heldIndex !== null;
}

export function startDrag(event: PointerEvent, index: number, folder: string | null): void {
  if (event.button !== 0) return; // Only allow left click drag
  if (isLocked()) return;

  heldIndex = index;
  heldFolder = folder;
  clearHover();
}

export function hover(event: PointerEvent, index: number, folder: string | null): void {
  if (heldIndex === null) return;

  // Releasing over the held row itself changes nothing
  if (isHeld(index, folder)) {
    clearHover();
    return;
  }

  // A folder can't go inside a folder
  if (heldIsFolder() && folder !== null) {
    clearHover();
    return;
  }

  const row = event.currentTarget as HTMLElement;
  const rect = row.getBoundingClientRect();

  hoveredIndex = index;
  hoveredFolder = folder;
  hoveredBelow = event.clientY > rect.top + rect.height / 2;
}

// Pointer over the empty space below the last row, the end of the top level
export function hoverEnd(): void {
  if (heldIndex === null) return;

  hoveredIndex = getLayout().length;
  hoveredFolder = null;
  hoveredBelow = false;
}

export function endDrag(): void {
  if (heldIndex === null || hoveredIndex === null) {
    cancelDrag();
    return;
  }

  const entry = getLayout()[hoveredIndex];
  if (isDropInto(hoveredIndex) && typeof entry !== "string") {
    moveEntry(heldIndex, heldFolder, entry.rooms.length, entry.name);
    cancelDrag();
    return;
  }

  let index = hoveredIndex;
  if (hoveredBelow) {
    index += 1;
  }

  // Taking the held row out first moves every row after it up by one
  if (heldFolder === hoveredFolder && heldIndex < index) {
    index -= 1;
  }

  moveEntry(heldIndex, heldFolder, index, hoveredFolder);
  cancelDrag();
}


// Also called when the browser ends the drag before release
export function cancelDrag(): void {
  heldIndex = null;
  heldFolder = null;
  clearHover();
}

// Checks if this row is the one being dragged. A row is found by its index
// and the folder its in (no folder means top level)
export function isHeld(index: number, parentFolder: string | null): boolean {
  return heldIndex === index && heldFolder === parentFolder;
}

// Null when nothing is being dragged over this row
export function dropMarkAt(index: number, folder: string | null): DropMark | null {
  if (!isHovered(index, folder)) return null;
  if (isDropInto(index)) return DropMark.Into;
  return hoveredBelow ? DropMark.Below : DropMark.Above;
}

// Whether the held room would go into the folder at this top level position
function isDropInto(index: number): boolean {
  if (heldIndex === null || heldIsFolder()) return false;
  return isHovered(index, null) && isFolderAt(index);
}

// Helper Methods //

// Checks if this row is the one under the pointer. A row is found by its
// index and the folder its in (no folder means top level)
function isHovered(index: number, folder: string | null): boolean {
  return hoveredIndex === index && hoveredFolder === folder;
}

function clearHover(): void {
  hoveredIndex = null;
  hoveredFolder = null;
  hoveredBelow = false;
}

function heldIsFolder(): boolean {
  return heldIndex !== null && heldFolder === null && isFolderAt(heldIndex);
}

function isFolderAt(index: number): boolean {
  const entry = getLayout()[index];
  return entry !== undefined && typeof entry !== "string";
}
