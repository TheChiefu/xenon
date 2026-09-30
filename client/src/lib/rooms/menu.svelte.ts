import { isLocked, lockLayout, unlockLayout } from "@/lib/rooms/layout.svelte";
import { startCreatingFolder } from "@/lib/rooms/folders.svelte";
import { getToken, getUrl } from "@/lib/session.svelte";

// Context menu position, undefined when closed
let menuPos: { x: number, y: number } | undefined = $state();

export function getMenuPos(): { x: number, y: number } | undefined {
  return menuPos;
}

export function openMenu(event: MouseEvent): void {
  event.preventDefault();
  menuPos = { x: event.clientX, y: event.clientY };
}

export function closeMenu(): void {
  menuPos = undefined;
}

export function onToggleLock(): void {
  const url = getUrl();
  const token = getToken();
  if (url === null || token === null) return;

  if (isLocked()) {
    unlockLayout();
  } else {
    lockLayout(url, token);
  }
  closeMenu();
}

export function onAddFolder(): void {
  startCreatingFolder();
  closeMenu();
}

export function onNewRoom(): void {
  // TODO: open the room creation dialog once it exists
  closeMenu();
}
