// Client Settings - Notifications

import { getToken, getUrl } from "@/lib/session.svelte";

let browserGranted: boolean = $state(checkBrowserGranted());
let pushSubscription: PushSubscription | null = $state(null);

// Helpers //

function checkBrowserGranted(): boolean {
  return Notification.permission === "granted";
}

// Decodes a base64url string into raw bytes
function decodeBase64Url(value: string): number[] {
  const base64 = value.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64.padEnd(base64.length + (4 - base64.length % 4) % 4, "=");
  const binary = atob(padded);
  return Array.from(binary, (char) => char.charCodeAt(0));
}

// Fetches the server's VAPID key
async function fetchVAPID(url: string): Promise<Uint8Array<ArrayBuffer>> {
  const response = await fetch(`${url}/push/vapid`, {method: "GET"});
  const data = await response.json();

  // Report any failures
  if (!response.ok) {
    throw new Error(data.error);
  }

  return new Uint8Array(data as number[]);
}

// Check for an existing push subscription
async function syncPushSubscription(): Promise<void> {
  const registration = await navigator.serviceWorker.getRegistration();
  pushSubscription = (await registration?.pushManager.getSubscription()) ?? null;
}
syncPushSubscription();

// Public //

export function isBrowserNotificationGranted(): boolean {
  return browserGranted;
}

export function isPushSubscribed(): boolean {
  return pushSubscription !== null;
}

// Prompts the browser's notification permission dialog.
// Returns an error message on failure, or undefined on success.
export async function enableBrowserNotifications(): Promise<string | undefined> {
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

// Registers the service worker, subscribes it to push using the server's VAPID key, and
// reports the subscription to the server so it has somewhere to send pushes.
// Returns an error message on failure, or undefined on success.
export async function enablePush(): Promise<string | undefined> {
  const url = getUrl();
  if (url === null) {
    return "Not signed in";
  }

  try {
    // Get VAPID key from server
    const applicationServerKey = await fetchVAPID(url);

    // Register service worker and subscribe to push
    const registration = await navigator.serviceWorker.register("/sw.js");
    const subscription = await registration.pushManager.subscribe({
      applicationServerKey,
      userVisibleOnly: true
    });

    // Send subscription to server
    const json = subscription.toJSON();
    const response = await fetch(`${url}/me/push`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${getToken()}`,
      },
      body: JSON.stringify({
        endpoint: json.endpoint!,
        p256dh: decodeBase64Url(json.keys!.p256dh),
        auth: decodeBase64Url(json.keys!.auth),
      }),
    });

    // If failed, return error
    if (!response.ok) {
      throw new Error((await response.json()).error);
    }

    // No failures, save subscription
    pushSubscription = subscription;
    return undefined;
  } catch (err) {
    // Report any unknown errors
    return err instanceof Error ? err.message : String(err);
  } finally {
    // Push can prompt for browser notifications
    // Check if user allowed them
    browserGranted = checkBrowserGranted();
  }
}

// Removes the subscription from the server, then unsubscribes it locally.
// Returns an error message on failure, or undefined on success.
export async function disablePush(): Promise<string | undefined> {
  // Nothing to unsubscribe from
  if (pushSubscription === null) {
    return undefined;
  }

  // No session to authenticate the DELETE request with
  const url = getUrl();
  if (url === null) {
    return "Not signed in";
  }

  try {
    // Delete existing push subscription
    const response = await fetch(`${url}/me/push`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${getToken()}`,
      },
      body: JSON.stringify({ endpoint: pushSubscription.endpoint }),
    });

    // If failed, return error
    if (!response.ok) {
      throw new Error((await response.json()).error);
    }

    // No failures, unsubscribe on local browser
    await pushSubscription.unsubscribe();
    pushSubscription = null;
    return undefined;
  } catch (err) {
    // Report any unknown errors
    return err instanceof Error ? err.message : String(err);
  }
}
