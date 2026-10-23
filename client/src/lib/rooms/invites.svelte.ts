import type { ReceivedInviteResponse } from "@/bindings/routes/rooms";
import * as roomsAPI from "@/lib/api/rooms";
import { LocalEvent, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
import { failureText } from "@/lib/utils";

let received: ReceivedInviteResponse[] = $state([]);
let pending: boolean = $state(false);
let error: string | null = $state(null);

export function getInvites(): ReceivedInviteResponse[] {
  return received;
}

export function isPending(): boolean {
  return pending;
}

export function getError(): string | null {
  return error;
}

export async function load(): Promise<boolean> {
  return run(async () => {
    received = await roomsAPI.listMyInvites();
  });
}

export async function accept(room_id: string): Promise<boolean> {
  return await run(() => roomsAPI.join(room_id)) && discard(room_id);
}

export async function decline(room_id: string): Promise<boolean> {
  return await run(() => roomsAPI.declineInvite(room_id)) && discard(room_id);
}

function discard(room_id: string): boolean {
  received = received.filter((entry) => entry.room_id !== room_id);
  return true;
}

function clear(): void {
  received = [];
  error = null;
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

events.subscribe(ServerEventType.invited, load);
events.subscribe(ServerEventType.resync, load);

events.subscribe(ServerEventType.invite_revoked, (event) => {
  discard(event.room_id);
});

events.subscribe(LocalEvent.signed_out, clear);
