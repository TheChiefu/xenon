// Modal dialogs shown over the app, one at a time
export enum Modal {
  settings = "settings",
  admin = "admin",
}

// State //
let current: Modal | null = $state(null);  // Open modal, null when none

// Getters //
export function isOpen(modal: Modal): boolean {
  return current === modal;
}

// Dialog //
export function open(modal: Modal): void {
  current = modal;
}
export function close(): void {
  current = null;
}
