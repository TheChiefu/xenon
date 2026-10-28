// Client Settings - Status this device presents to other users

import { Appearance } from "@/bindings/types/status";
import * as settings from "@/lib/client/settings.svelte";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

const keyStatus = "default_status";
const defaultStatus: Appearance = Appearance.online;
const activityEvents = ["pointerdown", "keydown", "wheel"];

let currentStatus: Appearance = $state(readPreferedStatus());
let isAutoAway = $state(false);
let awayTimer: number | undefined;

// Check for user activity on listener
for (const event of activityEvents) {
  window.addEventListener(event, onActivity, { passive: true });
}
document.addEventListener("visibilitychange", onVisible);
scheduleAway();

// STATUS //

export function get(): Appearance {
  if (isAutoAway && currentStatus === Appearance.online) {
    return Appearance.away;
  }
  return currentStatus;
}

// The preferred status on this device
export function getPreferred(): Appearance {
  return currentStatus;
}

export function set(status: Appearance): void {
  isAutoAway = false;
  currentStatus = status;
  localStorage.setItem(keyStatus, status); // Set status preference on client
  events.dispatch(LocalEvent.status_changed, get()); // Send server event about status
}

// IDLE //

function onActivity(): void {
  if (isAutoAway) {
    isAutoAway = false;
    events.dispatch(LocalEvent.status_changed, get());
  }

  scheduleAway();
}

function onVisible(): void {
  if (!document.hidden) {
    onActivity();
  }
}

function scheduleAway(): void {
  clearTimeout(awayTimer);

  // Check if user is away (0 means never idle)
  const minutes = settings.getAwayMinutes();
  if (minutes <= 0) {
    awayTimer = undefined;
    return;
  }

  awayTimer = setTimeout(onAway, minutes * 60_000);
}

function onAway(): void {
  if (currentStatus !== Appearance.online) {
    return;
  }

  isAutoAway = true;
  events.dispatch(LocalEvent.status_changed, get());
}

// STORAGE //

function readPreferedStatus(): Appearance {
  const stored = localStorage.getItem(keyStatus);
  const statuses: string[] = Object.values(Appearance);
  if (stored === null || !statuses.includes(stored)) {
    return defaultStatus;
  }
  return stored as Appearance;
}
