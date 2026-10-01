// Room panel width
const KEY_USER_ROOM_WIDTH: string = "prefRoomWidth";
const DEFAULT_WIDTH: number = 300;
export const HANDLE_WIDTH: number = 8;

const storedWidth: string | null = localStorage.getItem(KEY_USER_ROOM_WIDTH);

let width: number = $state(storedWidth !== null ? Number(storedWidth) : DEFAULT_WIDTH);
let resizing: boolean = $state(false);

// Getters and Setters 

export function getWidth(): number {
  return width;
}

export function isResizing(): boolean {
  return resizing;
}

export function reset(): void {
  width = DEFAULT_WIDTH;
  localStorage.setItem(KEY_USER_ROOM_WIDTH, String(width));
}


// Pointer Events //

export function resizeStart(event: PointerEvent): void {
  const handle = event.currentTarget as HTMLElement;
  handle.setPointerCapture(event.pointerId);
  resizing = true;
}

export function resize(event: PointerEvent, pane: HTMLElement | undefined): void {
  if (!resizing || pane === undefined) return;

  const left: number = pane.getBoundingClientRect().left;
  const next: number = event.clientX - left;
  width = Math.min(Math.max(next, HANDLE_WIDTH), window.innerWidth);
}

export function resizeStop(): void {
  if (!resizing) return;

  resizing = false;
  localStorage.setItem(KEY_USER_ROOM_WIDTH, String(width));
}
