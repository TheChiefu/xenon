import type { RespondAccess, Patch } from "@/bindings/routes/rooms";
import type { RoomInfo } from "@/bindings/types/rooms";
import { Permission, type Notify } from "@/bindings/types/rooms";
import * as roomsAPI from "@/lib/api/rooms";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import * as profile from "@/lib/users/profile.svelte";
import * as room from "@/lib/rooms/data.svelte";
import * as members from "@/lib/rooms/members.svelte";
import { failureText } from "@/lib/utils";

let room_id: string | null = $state(null);
let selected_id: string | null = $state(null);
let access: RespondAccess | null = $state(null);
let pending: boolean = $state(false);
let error: string | null = $state(null);

// ROOM //

// Gets the open room
export function getRoom(): RoomInfo | undefined {
  return room_id === null ? undefined : room.get(room_id);
}

// Updates the open room
export async function saveRoom(patch: Patch): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return run(() => roomsAPI.update(id, patch));
}

// Leaves the open room
export async function leave(): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return run(() => roomsAPI.leave(id));
}

// Deletes the open room
export async function deleteRoom(): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;
  return run(() => roomsAPI.remove(id));
}

// Clears room ID
export function close(): void {
  room_id = null;
}

// NOTIFY //

// Gets own notify setting for the open room
export function getNotify(): Notify | undefined {
  return room_id === null ? undefined : members.getNotify(room_id);
}

// Updates own notify setting for the open room
export async function saveNotify(notify: Notify): Promise<boolean> {
  const id = room_id;
  const self_id = profile.get()?.id;
  if (id === null || self_id === undefined) return false;

  return run(async () => {
    await roomsAPI.updateMember(id, self_id, { permissions: null, notify });
    members.setNotify(id, notify);
  });
}

// PERMISSIONS //

// Gets own permissions in the open room
export function getPermissions(): Permission[] {
  const self_id = profile.get()?.id;
  if (room_id === null || self_id === undefined) return [];
  return members.getPermissions(room_id, self_id);
}

// Fetches own permissions when not cached
async function loadOwnPermissions(id: string): Promise<void> {
  const self_id = profile.get()?.id;
  if (self_id === undefined || members.hasPermissions(id, self_id)) return;

  await run(async () => {
    const own = await roomsAPI.getMember(id, self_id);
    if (own.permissions !== null) {
      members.setPermissions(id, self_id, own.permissions);
    }
  });
}

// Replaces a member's permissions in the open room
export async function savePermissions(user_id: string, granted: Permission[]): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;

  return run(async () => {
    await roomsAPI.updateMember(id, user_id, { permissions: granted, notify: null });
    members.setPermissions(id, user_id, granted);

    if (access !== null) {
      access.permissions = granted;
    }
  });
}

// ACCESS //

// Gets the selected user's ID
export function getSelectedId(): string | null {
  return selected_id;
}

// Gets the selected user's access to the open room
export function getAccess(): RespondAccess | null {
  return access;
}

// Selects a user and fetches their access, or deselects on null
export async function select(user_id: string | null): Promise<boolean> {
  selected_id = user_id;
  access = null;

  const id = room_id;
  if (id === null || user_id === null) return false;

  return run(async () => {
    access = await roomsAPI.getMember(id, user_id);
  });
}

// Invites a user to the open room
export async function invite(user_id: string): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;

  return run(async () => {
    await roomsAPI.invite(id, user_id);

    if (access !== null) {
      access.invited = true;
    }
  });
}

// Revokes an invite the open room sent
export async function revokeInvite(user_id: string): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;

  return run(async () => {
    await roomsAPI.revokeInvite(id, user_id);

    if (access !== null) {
      access.invited = false;
    }
  });
}

// Bans a user from the open room
export async function ban(user_id: string): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;

  return run(async () => {
    await roomsAPI.ban(id, user_id);

    if (access !== null) {
      access.banned = true;
      access.permissions = null;
    }
  });
}

// Lifts a user's ban on the open room
export async function unban(user_id: string): Promise<boolean> {
  const id = room_id;
  if (id === null) return false;

  return run(async () => {
    await roomsAPI.unban(id, user_id);

    if (access !== null) {
      access.banned = false;
    }
  });
}

// STATUS //

// Whether a request is in flight
export function isPending(): boolean {
  return pending;
}

// Last request's failure text
export function getError(): string | null {
  return error;
}

// Runs a request, tracking pending and error
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

// LISTENERS //

// Open settings for a room
events.subscribe(LocalEvent.room_settings_requested, (id: string) => {
  room_id = id;
  selected_id = null;
  access = null;
  error = null;
  loadOwnPermissions(id);
});
