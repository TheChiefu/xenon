import type { RoomListItem } from "@/bindings/shared";
import { addRoomToFolder, getLayout, isLocked, reorderRoom } from "@/lib/rooms/layout.svelte";
import { isCollapsed } from "@/lib/rooms/folders.svelte";

// Row at this position is held, null when nothing is held
let heldIndex: number | null = $state(null);

// Pointer position when the row was first held
let startY: number = 0;

// Height of the held row, measured once when it's picked up
let rowHeight: number = 0;

// Current pointer position, tracked while held
let pointerY: number = $state(0);

export function isDragging(): boolean {
  return heldIndex !== null;
}

// Records the row and starting position a drag begins from
export function startDrag(event: PointerEvent, rowIndex: number): void {
  if (event.button !== 0) return; // Only allow left click drag
  if (isLocked()) return;

  const row = event.currentTarget as HTMLElement;
  heldIndex = rowIndex;
  startY = event.clientY;
  pointerY = event.clientY;
  rowHeight = row.getBoundingClientRect().height;
}

// Tracks the pointer while a row is held
export function trackDrag(event: PointerEvent): void {
  if (heldIndex === null) return;
  pointerY = event.clientY;
}

// Divides the distance moved by one row's height, then rounds to the
// nearest whole row, to find how many rows the pointer has crossed
export function currentIndex(): number {
  if (heldIndex === null) return 0;

  const crossed = heldIndex + Math.round((pointerY - startY) / rowHeight);
  return Math.max(0, crossed);
}

// Position for one row while a drag is in progress, null if it shouldn't move
export function getTransform(rowIndex: number): string | null {
  if (heldIndex === null) return null;

  // The held row follows the pointer directly
  if (rowIndex === heldIndex) {
    return `translateY(${pointerY - startY}px)`;
  }

  // Rows between the held row's start and current position shift by one row height
  const target = currentIndex();

  if (heldIndex < rowIndex && rowIndex <= target) {
    return `translateY(${-rowHeight}px)`;
  }

  if (target <= rowIndex && rowIndex < heldIndex) {
    return `translateY(${rowHeight}px)`;
  }

  return null;
}

// Commits the held row to its current position or into a folder
export function endDrag(): void {
  if (heldIndex === null) return;

  const target = currentIndex();
  const targetEntry = getLayout()[target];

  if (targetEntry === undefined || typeof targetEntry === "string") {
    reorderRoom(heldIndex, target);
  } else {
    addRoomToFolder(heldIndex, targetEntry.name);
  }

  heldIndex = null;
}

// Browser ended the drag before release, nothing is committed
export function cancelDrag(): void {
  heldIndex = null;
}