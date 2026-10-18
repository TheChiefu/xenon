import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

export interface Account {
  username: string;
  url: string;
  token: string;
  room: string | null;
}

// STORAGE //

const key_accounts = "accounts";
const key_logged_in = "logged_in";

const accounts_json = localStorage.getItem(key_accounts);
let accounts: Record<string, Account> = $state(accounts_json ? JSON.parse(accounts_json) : {});
let current_login: string | null = $state(localStorage.getItem(key_logged_in));

// ACCOUNTS //
function save(): void {
  localStorage.setItem(key_accounts, JSON.stringify(accounts));
}

export function getAccounts(): Record<string, Account> {
  return accounts;
}

export function addAccount(username: string, url: string, token: string): void {
  const key = `${username}@${url}`;
  let room: string | null = null;

  if (accounts[key] !== undefined) {
    room = accounts[key].room;
  }

  accounts[key] = { username, url, token, room };

  save();
  setCurrentLogin(key);
}

export function removeAccount(key: string): void {
  delete accounts[key];
  save();

  if (current_login === key) {
    clearCurrentLogin();
  }
}

export function removeLoggedInAccount(): void {
  if (current_login !== null) {
    removeAccount(current_login);
  }
}

// LOGIN //

export function getLoggedInKey(): string | null {
  return current_login;
}

// The current account logged in
export function getLoggedInAccount(): Account | null {
  if (current_login === null) { return null; }
  return accounts[current_login] ?? null;
}

export function setCurrentLogin(key: string): void {
  current_login = key;
  localStorage.setItem(key_logged_in, key);
}

export function clearCurrentLogin(): void {
  current_login = null;
  localStorage.removeItem(key_logged_in);
  events.dispatch(LocalEvent.signed_out);
}

// REQUESTS //

// Token that requests are sent with
export function getToken(): string | null {
  const account = getLoggedInAccount();
  if (account === null) { return null; }
  return account.token;
}

export function getUrl(): string | null {
  const account = getLoggedInAccount();
  if (account === null) { return null; }
  return account.url;
}

// ROOM SELECTION //

export function getSelectedRoom(): string | null {
  const account = getLoggedInAccount();
  if (account === null) { return null;}
  return account.room ?? null;
}

export function selectRoom(id: string | null): void {
  const account = getLoggedInAccount();
  if (account === null) { return;}
  account.room = id;
  save();
}

events.subscribe(LocalEvent.rooms_changed, (room_ids: string[]) => {
  const selected = getSelectedRoom();
  if (selected !== null && !room_ids.includes(selected)) {
    selectRoom(null);
  }
});
