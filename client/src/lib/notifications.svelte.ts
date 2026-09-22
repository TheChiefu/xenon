// Client Settings - Notifications

import type { Subscription } from "@/bindings/sockets/events";
import { getToken } from "@/lib/session.svelte";
import { parseErrorResponse } from "@/lib/error";

let browserGranted: boolean = $state(Notification.permission === "granted");
let pushGranted: boolean = $state(false);

// Whether the browser has granted notification permission
export function getBrowserNotificationsEnabled(): boolean {
  return browserGranted;
}

// Prompts the browser's notification permission dialog.
// Returns an error message on failure, or undefined on success.
export async function requestBrowserNotifications(): Promise<string | undefined> {
  const result = await Notification.requestPermission();

  switch (result) {
    case "granted":
      browserGranted = true;
      return undefined;
    case "denied":
      return "Browser notification request: Denied";
    case "default":
      return "Browser notification request: not found";
  }
}

export function getPushStates(): boolean {
  return true;
}

// Fetches the server's VAPID key
async function fetchVAPID(url: string): Promise<Uint8Array> {
  const response = await fetch(`${url}/push/vapid`, {method: "GET"});
  const data = await response.json();

  if (!response.ok) {
    const errorResponse = parseErrorResponse(data);
    if (errorResponse === null) {
      throw new Error("Unexpected error response from /push/vapid");
    }
    throw new Error(errorResponse.error);
  }

  return new Uint8Array(data as number[]);
}

// Registers the service worker and subscribes it to push using the server's VAPID key
export async function subscribeToPush(url: string): Promise<PushSubscription | undefined> {
  const applicationServerKey = await fetchVAPID(url);
  const registration = await navigator.serviceWorker.register("/sw.js"); // Lazy register to service worker at web root

  if (registration) {
    return await registration.pushManager.subscribe({
      applicationServerKey,
      userVisibleOnly: true
    })
  }

  return undefined;
}