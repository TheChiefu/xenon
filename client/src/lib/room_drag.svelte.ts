import { moveRoom } from "@/lib/rooms.svelte";

// Vertical travel before a held room is picked up, so a click still selects
const MOVE_THRESHOLD: number = 5;

let fromIndex: number | null = $state(null);
let pressY: number = 0;

// Room boxes, measured on pickup before any shifting
let roomRects: DOMRect[] = [];
let pointerY: number = $state(0);
let moving: boolean = $state(false);

export function isMoving(): boolean {
  return moving;
}

export function isHeld(index: number): boolean {
  return moving && fromIndex === index;
}

// Held room follows the pointer, rooms between it and the drop position step aside
// Null when the room sits in place
export function getRoomTransform(index: number): string | null {
  if (!moving || fromIndex === null) return null;

  const drop: number = dropIndex();
  const height: number = roomRects[fromIndex].height;

  let shift: number = 0;
  if (index === fromIndex) {
    let center = roomRects[fromIndex].top + (roomRects[fromIndex].height / 2);
    shift = pointerY - center;
  } else if (fromIndex < index && index <= drop) {
    shift = -height;
  } else if (drop <= index && index < fromIndex) {
    shift = height;
  }

  return shift !== 0 ? `translateY(${shift}px)` : null;
}

// Hold a room, picked up once past the threshold
export function holdRoom(event: PointerEvent, index: number): void {
  if (event.button !== 0) return; // Only allow left click drag-drop
  fromIndex = index;
  pressY = event.clientY;
  moving = false;
}

// Pick up past the threshold, then follow the pointer
// List takes the capture on pickup, so moves off the list still arrive
export function dragRoom(event: PointerEvent, list: HTMLElement | undefined): void {
  if (fromIndex === null || list === undefined) return;

  // Released off the list before pickup
  if (event.buttons === 0) {
    fromIndex = null;
    return;
  }

  if (!moving) {
    if (Math.abs(event.clientY - pressY) < MOVE_THRESHOLD) return;

    list.setPointerCapture(event.pointerId);
    roomRects = [];
    for (const room of list.children) {
      roomRects.push(room.getBoundingClientRect());
    }
    moving = true;
  }

  pointerY = event.clientY;
}

// Move held room to the drop position (on release)
export function dropRoom(): void {
  if (fromIndex !== null && moving) {
    moveRoom(fromIndex, dropIndex());
  }
  fromIndex = null;
  moving = false;
}

// Browser ended drag before release, leave room in place
export function cancelRoom(): void {
  fromIndex = null;
  moving = false;
}


// Helper Methods //

// Drop position, count of other rooms centered above the pointer
function dropIndex(): number {
  let count: number = 0;
  for (let i = 0; i < roomRects.length; i++) {
    if (i === fromIndex) continue; // Held room's own slot doesn't count

    const center: number = roomRects[i].top + roomRects[i].height / 2;
    if (center < pointerY) {
      count++;
    }
}
  return count;
}
