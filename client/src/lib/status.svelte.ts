// Client Settings - Status this device presents to other users

import { Status } from "@/bindings/shared";
import { LABELS_STATUS } from "@/lib/labels";
import { getAwayMinutes } from "@/lib/settings.svelte";

const keyStatus = "prefStatus";
const defaultStatus: Status = Status.online;
const idleCheckMs = 15_000;
const activityEvents = ["pointerdown", "keydown", "wheel"];

let currentStatus: Status = $state(readStored());
let idle = $state(false);
let lastActivity = Date.now();

for (const event of activityEvents) {
  window.addEventListener(event, onActivity, { passive: true });
}

setInterval(checkIdle, idleCheckMs);

// The status chosen on this device
export function getStatus(): Status {
  if (idle && currentStatus === Status.online) {
    return Status.away;
  }

  return currentStatus;
}

// Set the status and remember it on this device
export function setStatus(status: Status): void {
  currentStatus = status;
  localStorage.setItem(keyStatus, status);

  // Telling the server needs the socket, which the client does not have yet
}

function onActivity(): void {
  lastActivity = Date.now();
  idle = false;
}

function checkIdle(): void {
  const minutes = getAwayMinutes();

  // 0 never goes away
  idle = minutes > 0 && Date.now() - lastActivity >= minutes * 60_000;
}

function readStored(): Status {
  const stored = localStorage.getItem(keyStatus);

  // Anything can be written into storage, so an unknown name falls back
  if (stored === null || !(stored in LABELS_STATUS)) {
    return defaultStatus;
  }

  return stored as Status;
}
