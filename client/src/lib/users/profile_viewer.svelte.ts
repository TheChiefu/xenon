import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

// State //
let visible: boolean = $state(false);      // Dialog is on screen
let userId: string | null = $state(null);  // User whose profile is shown

// Getters //

export function isOpen(): boolean {
  return visible;
}

export function getUserId(): string | null {
  return userId;
}

// Dialog //

export function close(): void {
  visible = false;
}

export function open(id: string | null): void {
  if (id === null) {
    return;
  }

  userId = id;
  visible = true;
}

// Open the profile of a selected user
events.subscribe(LocalEvent.user_selected, open);
