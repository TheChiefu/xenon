// Client Settings - Status this device presents to other users

import { Status } from "@/bindings/shared";
import { getAwayMinutes } from "@/lib/settings.svelte";
import { send } from "@/lib/socket.svelte";

const keyStatus = "default_status";
const defaultStatus: Status = Status.online;
const idleCheckDelayMS = 15_000;
const activityEvents = ["pointerdown", "keydown", "wheel"];

let currentStatus: Status = $state(readPreferedStatus());
let isIdle = $state(false);
let lastActiveTime = Date.now();

// Check for user activity on listener, check activity every set interval time
for (const event of activityEvents) {
  window.addEventListener(event, onActivity, { passive: true });
}
setInterval(checkIdle, idleCheckDelayMS);

// STATUS //

// The preferred status on this device
export function getStatus(): Status {
  if (isIdle && currentStatus === Status.online) {
    return Status.away;
  }
  return currentStatus;
}

export function setStatus(status: Status): void {
  currentStatus = status;
  localStorage.setItem(keyStatus, status); // Set status preference on client
  send({ type: "status", status: getStatus() }); // Send server event about status
}

// IDLE //

// Updates when the last user interaction was made (for idle timer)
function onActivity(): void {
  lastActiveTime = Date.now();
}

function checkIdle(): void {
  const minutes = getAwayMinutes();
  const elapsed = Date.now() - lastActiveTime;

  // Check if user is away (0 means never idle)
  const timePassed = (minutes > 0) && (elapsed >= minutes * 60_000);

  // If the state changed since last activity, update idle check and send server event
  if (timePassed !== isIdle) {
    isIdle = timePassed;
    send({ type: "status", status: getStatus() });
  }
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
