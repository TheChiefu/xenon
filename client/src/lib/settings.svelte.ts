// Client Settings - Adjustable by user

import darkUrl from "@/styles/themes/dark.css?url";
import lightUrl from "@/styles/themes/light.css?url";

// THEMES //
const keyTheme = "theme";
const defaultTheme = "dark";
const themeStylesheetId = "theme-stylesheet";

const builtinThemes: Record<string, string> = {
  dark: darkUrl,
  light: lightUrl,
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
