import type { BanResponse, IssuedInviteResponse, RoomPatch, RoomResponse } from "@/bindings/routes/rooms";
import type { Notify, Permission } from "@/bindings/shared";
import * as roomsAPI from "@/lib/api/rooms";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import * as profile from "@/lib/users/profile.svelte";
import * as room from "@/lib/rooms/data.svelte";
import * as members from "@/lib/rooms/members.svelte";
import * as users from "@/lib/users/data.svelte";
import { failureText } from "@/lib/utils";

let room_id: string | null = $state(null);
let invites: IssuedInviteResponse[] = $state([]);
let bans: BanResponse[] = $state([]);
let pending: boolean = $state(false);
let error: string | null = $state(null);

export function getRoom(): RoomResponse | undefined {
  return room_id === null ? undefined : room.get(room_id);
}

export function getPermissions(): Permission[] {
  const self_id = profile.get()?.id;
  if (room_id === null || self_id === undefined) return [];
  return members.getPermissions(room_id, self_id);
}

export function getNotify(): Notify | undefined {
  return room_id === null ? undefined : members.getNotify(room_id);
}

export function getInvites(): IssuedInviteResponse[] {
  return invites;
}

export function getBans(): BanResponse[] {
  return bans;
}

export function isPending(): boolean {
  return pending;
}

export function getError(): string | null {
  return error;
}

export async function saveRoom(patch: RoomPatch): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return run(() => roomsAPI.update(id, patch));
}

export async function saveNotify(notify: Notify): Promise<boolean> {
  const id = room_id;
  const self_id = profile.get()?.id;
  if (id === null || self_id === undefined) return false;

  return run(async () => {
    await roomsAPI.updateMember(id, self_id, { permissions: null, notify });
    members.setNotify(id, notify);
  });
}

export async function leave(): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return run(() => roomsAPI.leave(id));
}

export async function deleteRoom(): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return run(() => roomsAPI.remove(id));
}

export async function loadInvites(): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;

  return run(async () => {
    invites = await roomsAPI.listInvites(id);
    await users.load(invites.flatMap((entry) => [entry.user_id, entry.invited_by]));
  });
}

export async function invite(user_id: string): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return await run(() => roomsAPI.invite(id, user_id, null)) && await loadInvites();
}

export async function revokeInvite(user_id: string): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return await run(() => roomsAPI.revokeInvite(id, user_id)) && await loadInvites();
}

export async function loadBans(): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;

  return run(async () => {
    bans = await roomsAPI.listBans(id);
    await users.load(bans.flatMap((entry) => [entry.user_id, entry.created_by]));
  });
}

export async function unban(user_id: string): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return await run(() => roomsAPI.unban(id, user_id)) && await loadBans();
}

async function run(action: () => Promise<void>): Promise<boolean> {
  pending = true;
  error = null;

  try {
    await action();
    return true;
  } catch (err) {
    error = failureText(err);
    return false;
  } finally {
    pending = false;
  }
}

events.subscribe(LocalEvent.room_settings_requested, (id: string) => {
  room_id = id;
  invites = [];
  bans = [];
  error = null;
  members.load(id);
});
