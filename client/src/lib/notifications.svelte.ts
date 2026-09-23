// Client Settings - Notifications

import { getToken } from "@/lib/session.svelte";

let browserGranted: boolean = $state(Notification.permission === "granted");
let pushGranted: boolean = $state(false);

// Helpers //

// Decodes a base64url string into raw bytes
function decodeBase64Url(value: string): number[] {
  const base64 = value.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64.padEnd(base64.length + (4 - base64.length % 4) % 4, "=");
  const binary = atob(padded);
  return Array.from(binary, (char) => char.charCodeAt(0));
}

// Returns the browser's push subscription, or null if it doesn't hold one
async function getExistingSubscription(): Promise<PushSubscription | null> {
  const registration = await navigator.serviceWorker.getRegistration();
  const subscription = await registration?.pushManager.getSubscription();
  return subscription ?? null;
};


// Fetches the server's VAPID key
async function fetchVAPID(url: string): Promise<Uint8Array<ArrayBuffer>> {
  const response = await fetch(`${url}/push/vapid`, {method: "GET"});
  const data = await response.json();

  if (!response.ok) {
    throw new Error(data.error);
  }

  return new Uint8Array(data as number[]);
}

// Public Methods //

export function browserNotificationsOn(): boolean {
  return browserGranted;
}

export function pushNotificationsOn(): boolean {
  return pushGranted;
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

// Registers the service worker, subscribes it to push using the server's VAPID key, and
// reports the subscription to the server so it has somewhere to send pushes
export async function subscribeToPush(url: string): Promise<PushSubscription> {
  const applicationServerKey = await fetchVAPID(url);

  const registration = await navigator.serviceWorker.register("/sw.js"); // Lazy register to service worker at web root
  const subscription = await registration.pushManager.subscribe({
    applicationServerKey,
    userVisibleOnly: true
  });

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

  if (!response.ok) {
    throw new Error((await response.json()).error);
  }

  pushGranted = true;
  return subscription;
}

// Removes a subscription from the server, then unsubscribes it locally
export async function unsubscribeFromPush(url: string, subscription: PushSubscription): Promise<void> {
  const response = await fetch(`${url}/me/push`, {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${getToken()}`,
    },
    body: JSON.stringify({ endpoint: subscription.endpoint }),
  });

  if (!response.ok) {
    throw new Error((await response.json()).error);
  }

  await subscription.unsubscribe();
  pushGranted = false;
}
