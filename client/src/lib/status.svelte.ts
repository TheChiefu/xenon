// Client Settings - Status this device presents to other users

import { Status } from "@/bindings/shared";
import * as settings from "@/lib/settings.svelte";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

const keyStatus = "default_status";
const defaultStatus: Status = Status.online;
const activityEvents = ["pointerdown", "keydown", "wheel"];

let currentStatus: Status = $state(readPreferedStatus());
let isAutoAway = $state(false);
let awayTimer: number | undefined;

// Check for user activity on listener
for (const event of activityEvents) {
  window.addEventListener(event, onActivity, { passive: true });
}
document.addEventListener("visibilitychange", onVisible);
scheduleAway();

// STATUS //

export function get(): Status {
  if (isAutoAway && currentStatus === Status.online) {
    return Status.away;
  }
  return currentStatus;
}

// The preferred status on this device
export function getPreferred(): Status {
  return currentStatus;
}

export function set(status: Status): void {
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
  if (currentStatus !== Status.online) {
    return;
  }

  isAutoAway = true;
  events.dispatch(LocalEvent.status_changed, get());
}

// STORAGE //

function readPreferedStatus(): Status {
  const stored = localStorage.getItem(keyStatus);
  const statuses: string[] = Object.values(Status);
  if (stored === null || !statuses.includes(stored)) {
    return defaultStatus;
  }
  return stored as Status;
}
