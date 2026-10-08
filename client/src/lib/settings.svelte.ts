// Client Settings - Adjustable by user

import darkUrl from "@/styles/themes/dark.css?url";
import lightUrl from "@/styles/themes/light.css?url";
import sporeUrl from "@/styles/themes/spore.css?url";

// THEMES //
const keyTheme = "prefTheme";
const defaultTheme = "dark";
const themeStylesheetId = "theme-stylesheet";

const builtinThemes: Record<string, string> = {
  dark: darkUrl,
  light: lightUrl,
  spore: sporeUrl
};

let currentTheme: string = $state(localStorage.getItem(keyTheme) ?? defaultTheme);

// Swap the theme stylesheet to the given theme name
function applyTheme(theme: string): void {
  const url = builtinThemes[theme];
  if (url === undefined) {
    return;
  }

  // Find the existing theme <link>, or create one on first run
  let link = document.getElementById(themeStylesheetId) as HTMLLinkElement | null;
  if (link === null) {
    link = document.createElement("link");
    link.id = themeStylesheetId;
    link.rel = "stylesheet";
    document.head.appendChild(link);
  }

  link.href = url;
}

applyTheme(currentTheme);

// The currently active theme name
export function getTheme(): string {
  return currentTheme;
}

// Set the active theme and persist the selection
export function setTheme(theme: string): void {
  currentTheme = theme;
  localStorage.setItem(keyTheme, theme);
  applyTheme(theme);
}

// ANIMATE PHOTOS //
export enum AnimatePhotos {
  Always = "always",
  OnHover = "on-hover",
  Never = "never",
}

const keyAnimatePhotos = "prefAnimatePhotos";
const defaultAnimatePhotos: AnimatePhotos = AnimatePhotos.Always;

let currentAnimatePhotos: AnimatePhotos = $state(readAnimatePhotos());

export function getAnimatePhotos(): AnimatePhotos {
  return currentAnimatePhotos;
}

export function setAnimatePhotos(value: AnimatePhotos): void {
  currentAnimatePhotos = value;
  localStorage.setItem(keyAnimatePhotos, value);
}

// Attachment: plays or pauses a video to match the setting
export function animatePhoto(video: HTMLVideoElement): void {
  video.muted = true;
  video.loop = true;
  video.playsInline = true;
  video.disablePictureInPicture = true;
  video.style.pointerEvents = "none";

  if (currentAnimatePhotos === AnimatePhotos.Always) {
    video.play().catch(() => {});  // Rejects when paused before starting
  } else {
    video.pause();
  }
}

export function playOnHover(element: HTMLElement): () => void {
  const play = () => {
    element.querySelectorAll("video").forEach((video) => {
      video.play().catch(() => {});
    });
  };

  const pause = () => {
    element.querySelectorAll("video").forEach((video) => {
      video.pause();
    });
  };

  if (currentAnimatePhotos === AnimatePhotos.OnHover) {
    element.addEventListener("pointerenter", play);
    element.addEventListener("pointerleave", pause);
  }

  return () => {
    element.removeEventListener("pointerenter", play);
    element.removeEventListener("pointerleave", pause);
  };
}

function readAnimatePhotos(): AnimatePhotos {
  const stored = localStorage.getItem(keyAnimatePhotos);
  const values: string[] = Object.values(AnimatePhotos);

  // Fallback on unknown
  if (stored === null || !values.includes(stored)) {
    return defaultAnimatePhotos;
  }

  return stored as AnimatePhotos;
}

// AWAY TIMER //
const keyAwayMinutes = "prefAwayMinutes";
const defaultAwayMinutes = 10;

let currentAwayMinutes: number = $state(readAwayMinutes());

export function getAwayMinutes(): number {
  return currentAwayMinutes;
}

export function setAwayMinutes(minutes: number): void {
  if (!isValidAwayMinutes(minutes)) {
    return;
  }

  currentAwayMinutes = minutes;
  localStorage.setItem(keyAwayMinutes, String(minutes));
}

function isValidAwayMinutes(minutes: number): boolean {
  return Number.isFinite(minutes) && minutes >= 0;
}

function readAwayMinutes(): number {
  const stored = Number(localStorage.getItem(keyAwayMinutes) ?? defaultAwayMinutes);

  // Fallback on invalid
  if (!isValidAwayMinutes(stored)) {
    return defaultAwayMinutes;
  }

  return stored;
}
