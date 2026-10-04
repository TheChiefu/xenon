// Client Settings - Status this device presents to other users

import { Status } from "@/bindings/types";
import { LABELS_STATUS } from "@/lib/labels";

const keyStatus = "prefStatus";
const defaultStatus: Status = Status.Online;

let currentStatus: Status = $state(readStored());

// The status chosen on this device
export function getStatus(): Status {
  return currentStatus;
}

// Set the status and remember it on this device
export function setStatus(status: Status): void {
  currentStatus = status;
  localStorage.setItem(keyStatus, status);

  // Telling the server needs the socket, which the client does not have yet
}

function readStored(): Status {
  const stored = localStorage.getItem(keyStatus);

  // Anything can be written into storage, so an unknown name falls back
  if (stored === null || !(stored in LABELS_STATUS)) {
    return defaultStatus;
  }

  return stored as Status;
}
