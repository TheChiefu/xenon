// The other participants' screen shares, and the one being watched

import { SvelteMap } from "svelte/reactivity";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

// A participant's screen share
interface Screen {
  stream: MediaStream | null;
  sharing: boolean;
}

// STATE //

// The participant whose screen share is shown
let watching: string | null = $state(null);

// Screen shares, keyed by user id
const screens: SvelteMap<string, Screen> = new SvelteMap();

// GETTERS //

export function getWatching(): string | null {
  return watching;
}

export function isSharing(user_id: string): boolean {
  return screens.get(user_id)?.sharing ?? false;
}

export function getWatchedScreen(): MediaStream | null {
  if (watching === null) {
    return null;
  }

  return screens.get(watching)?.stream ?? null;
}

// WATCHING //

// Shows a participant's screen share, or stops showing it when already shown
export function watch(user_id: string): void {
  if (watching === user_id) {
    watching = null;
  } else {
    watching = user_id;
  }
}

export function stopWatching(): void {
  watching = null;
}

// Keeps a participant's screen share stream for when they share
export function add(user_id: string, stream: MediaStream): void {
  screens.set(user_id, { stream, sharing: isSharing(user_id) });
}

export function stop(user_id: string): void {
  screens.delete(user_id);

  if (watching === user_id) {
    watching = null;
  }
}

export function clear(): void {
  screens.clear();
  watching = null;
}

// LISTENERS //

// A participant starts or stops sharing
events.subscribe(LocalEvent.screen_shared, (event) => {
  const stream = screens.get(event.user_id)?.stream ?? null;
  screens.set(event.user_id, { stream, sharing: event.sharing });

  if (!event.sharing && watching === event.user_id) {
    watching = null;
  }
});
