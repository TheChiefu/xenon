// Room panel width
const KEY_USER_ROOM_WIDTH: string = "prefRoomWidth";
const DEFAULT_WIDTH: number = 200;

const storedWidth: string | null = localStorage.getItem(KEY_USER_ROOM_WIDTH);

let width: number = $state(storedWidth !== null ? Number(storedWidth) : DEFAULT_WIDTH);
let resizing: boolean = $state(false);

export function getWidth(): number {
  return width;
}

export function isResizing(): boolean {
  return resizing;
}

// Routed to handle until release
export function startResize(event: PointerEvent): void {
  const handle = event.currentTarget as HTMLElement;
  handle.setPointerCapture(event.pointerId);
  resizing = true;
}

// Follow pointer while resize in progress
export function resize(event: PointerEvent, pane: HTMLElement | undefined): void {
  if (!resizing || pane === undefined) return;

  const left: number = pane.getBoundingClientRect().left;
  const next: number = event.clientX - left;
  width = Math.min(Math.max(next, 0), window.innerWidth);
}

// End resize on release (or drop capture)
// Store result in local storage
export function endResize(): void {
  if (!resizing) return;

  resizing = false;
  localStorage.setItem(KEY_USER_ROOM_WIDTH, String(width));
}

// Reset to the default width
export function resetWidth(): void {
  width = DEFAULT_WIDTH;
  localStorage.setItem(KEY_USER_ROOM_WIDTH, String(width));
}
