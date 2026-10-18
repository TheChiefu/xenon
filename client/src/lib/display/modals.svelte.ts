import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

// Modal dialogs shown over the app, one at a time
export enum Modal {
  settings = "settings",
  admin = "admin",
  discover = "discover",
  room_settings = "room_settings",
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

// Subscriptions
events.subscribe(LocalEvent.signed_out, close); // Reset modal state on signout
events.subscribe(LocalEvent.discover_requested, () => open(Modal.discover));
events.subscribe(LocalEvent.settings_requested, () => open(Modal.settings));
events.subscribe(LocalEvent.admin_requested, () => open(Modal.admin));
events.subscribe(LocalEvent.room_settings_requested, () => open(Modal.room_settings));
