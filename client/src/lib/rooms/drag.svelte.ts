import type { RoomListItem } from "@/bindings/shared";
import { Drop, getLayout, isLocked, moveEntry } from "@/lib/rooms/layout.svelte";

// The held row, null when nothing is held
let held: RoomListItem | null = $state(null);

// The row under the pointer, null when releasing changes nothing
let hovered: RoomListItem | null = $state(null);

// Which third of the hovered row the pointer is in
let hoveredDrop: Drop = $state(Drop.Above);

// Pointer Events //

export function startDrag(event: PointerEvent, row: RoomListItem): void {
  if (event.button !== 0) return; // Only allow left click drag
  if (isLocked()) return;

  held = row;
  hovered = null;
}

export function hover(event: PointerEvent, row: RoomListItem): void {
  if (held === null) return;

  const element = event.currentTarget as HTMLElement;
  const rect = element.getBoundingClientRect();
  const at = (event.clientY - rect.top) / rect.height;

  hovered = row;

  if (at < 1 / 3) {
    hoveredDrop = Drop.Above;
  } else if (at > 2 / 3) {
    hoveredDrop = Drop.Below;
  } else {
    hoveredDrop = Drop.Inside;
  }
}

// Pointer over the empty space below the last row
export function hoverEnd(): void {
  if (held === null) return;

  const rows = getLayout();
  hovered = rows[rows.length - 1] ?? null;
  hoveredDrop = Drop.Below;
}

export function endDrag(): void {
  if (held === null || hovered === null) {
    cancelDrag();
    return;
  }

  moveEntry(held, hovered, hoveredDrop);
  cancelDrag();
}

// Also called when the browser ends the drag before release
export function cancelDrag(): void {
  held = null;
  hovered = null;
}

// Drag State //

export function isDragging(): boolean {
  return held !== null;
}

export function isHeld(row: RoomListItem): boolean {
  return held === row;
}

// Null when nothing is being dragged over this row
export function dropAt(row: RoomListItem): Drop | null {
  if (hovered !== row) return null;
  return hoveredDrop;
}
