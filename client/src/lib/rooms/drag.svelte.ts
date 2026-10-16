import type { RoomListItem } from "@/bindings/shared";
import { Section } from "@/lib/rooms/layout.svelte";
import * as layout from "@/lib/rooms/layout.svelte";

let held: RoomListItem | null = $state(null);    // Row being held
let hover: RoomListItem | null = $state(null);   // Row being hovered over
let hoverPart: Section = $state(Section.Above);  // Which part pointer is over

// Pointer Events //

// Capture row being held on initial drag start
export function start(event: PointerEvent, row: RoomListItem): void {
  if (event.button !== 0) return; // Only allow left click drag
  if (layout.isLocked()) return;
  held = row;
  hover = null;
}

// Determine drag state while holding row
export function hold(event: PointerEvent): void {
  if (held === null) return;

  const element = document
    .elementFromPoint(event.clientX, event.clientY)
    ?.closest<HTMLElement>("[data-room], [data-folder]");
  if (!element) return;

  const row = rowOf(element);
  if (row === undefined) return;
  hover = row;

  // Calculate which part of element the mouse is over (Y-Axis)
  const rect = element.getBoundingClientRect();
  const at = (event.clientY - rect.top) / rect.height;

  // Determine which section of the row is being hovered
  if (at < 0.33) {
    hoverPart = Section.Above;
  } else if (at > 0.66) {
    hoverPart = Section.Below;
  } else {
    hoverPart = Section.Inside;
  }
}

// Determine how to handle drag result
export function release(): void {

  // Invalid state is canceled
  if (held === null || hover === null) {
    reset();
    return;
  }

  // Move layout depending on move type
  layout.moveEntry(held, hover, hoverPart);
  reset();
}

// Drag State //
export function reset(): void {
  held = null;
  hover = null;
}

export function isActive(): boolean {
  return held !== null;
}

export function isHeld(row: RoomListItem): boolean {
  return held === row;
}

// Null when nothing is being dragged over this row
export function dropAt(row: RoomListItem): Section | null {
  if (hover !== row) return null;
  return hoverPart;
}
