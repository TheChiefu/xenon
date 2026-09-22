export interface Login {
  username: string;
  url: string;
  token: string;
}

const keyLogins = "logins";
const keyActiveKey = "activeLoginKey";

const storedLogins = localStorage.getItem(keyLogins);
let logins: Record<string, Login> = $state(storedLogins ? JSON.parse(storedLogins) : {});
let activeKey: string | null = $state(localStorage.getItem(keyActiveKey));

function loginKey(username: string, url: string): string {
  return `${username}@${url}`;
}

// The active login entry, or null if nothing is active
export function getActiveLogin(): Login | null {
  if (activeKey === null) return null;
  return logins[activeKey] ?? null;
}

// The token currently in use, or null if nothing is active
export function getToken(): string | null {
  if (activeKey === null) {
    return null;
  }

  const login = logins[activeKey];
  if (login === undefined) {
    return null;
  }

  return login.token;
}

// The URL the active login belongs to, or null if nothing is active
export function getUrl(): string | null {
  if (activeKey === null) {
    return null;
  }

  const login = logins[activeKey];
  if (login === undefined) {
    return null;
  }

  return login.url;
}

// Every saved login
export function getLogins(): Record<string, Login> {
  return logins;
}

// Save a login and make it the active one, replacing any existing entry for
// the same username and URL
export function addLogin(username: string, url: string, token: string): void {
  const key = loginKey(username, url);
  logins[key] = { username, url, token };
  activeKey = key;

  localStorage.setItem(keyLogins, JSON.stringify(logins));
  localStorage.setItem(keyActiveKey, activeKey);
}

// Make keyed login the active one
export function resumeLogin(key: string): void {
  activeKey = key;
  localStorage.setItem(keyActiveKey, key);
}

// Remove a saved login
// Local only, nothing is revoked server-side (no endpoint for it exists yet)
export function removeLogin(key: string): void {
  delete logins[key];
  localStorage.setItem(keyLogins, JSON.stringify(logins));

  if (activeKey === key) {
    activeKey = null;
    localStorage.removeItem(keyActiveKey);
  }
}

// Sign out of the active login
// Local only, nothing is revoked server-side (no endpoint for it exists yet)
export function signOut(): void {
  activeKey = null;
  localStorage.removeItem(keyActiveKey);
}
