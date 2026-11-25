// Client Settings - Appearance this device presents to other users

import { Appearance } from "@/bindings/types/appearance";
import * as settings from "@/lib/client/settings.svelte";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

const keyAppearance = "default_appearance";
const defaultAppearance: Appearance = Appearance.online;
const activityEvents = ["pointerdown", "keydown", "wheel"];

let currentAppearance: Appearance = $state(readPreferredAppearance());
let isAutoAway = $state(false);
let awayTimer: number | undefined;

// Check for user activity on listener
for (const event of activityEvents) {
  window.addEventListener(event, onActivity, { passive: true });
}
document.addEventListener("visibilitychange", onVisible);
scheduleAway();

// APPEARANCE //

export function get(): Appearance {
  if (isAutoAway && currentAppearance === Appearance.online) {
    return Appearance.away;
  }
  return currentAppearance;
}

// The preferred appearance on this device
export function getPreferred(): Appearance {
  return currentAppearance;
}

export function set(appearance: Appearance): void {
  isAutoAway = false;
  currentAppearance = appearance;
  localStorage.setItem(keyAppearance, appearance); // Set appearance preference on client
  events.dispatch(LocalEvent.appearance_changed, get()); // Send server event about appearance
}

// IDLE //

function onActivity(): void {
  if (isAutoAway) {
    isAutoAway = false;
    events.dispatch(LocalEvent.appearance_changed, get());
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
  if (currentAppearance !== Appearance.online) {
    return;
  }

  isAutoAway = true;
  events.dispatch(LocalEvent.appearance_changed, get());
}

// STORAGE //

function readPreferredAppearance(): Appearance {
  const stored = localStorage.getItem(keyAppearance);
  const appearances: string[] = Object.values(Appearance);
  if (stored === null || !appearances.includes(stored)) {
    return defaultAppearance;
  }
  return stored as Appearance;
}
