import type { DiscoverableRoomResponse } from "@/bindings/routes/rooms";
import type { ServerInfo } from "@/bindings/routes/server";
import * as roomsAPI from "@/lib/api/rooms";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import { failureText } from "@/lib/utils";

let listed: DiscoverableRoomResponse[] = $state([]);
let final_page: boolean = $state(true);
let pending: boolean = $state(false);
let error: string | null = $state(null);
let joined: Set<string> = $state(new Set());
let page_size: number = 1;

export function getRooms(): DiscoverableRoomResponse[] {
  return listed;
}

export function isFinalPage(): boolean {
  return final_page;
}

export function isPending(): boolean {
  return pending;
}

export function getError(): string | null {
  return error;
}

export function isJoined(room_id: string): boolean {
  return joined.has(room_id);
}

export async function loadFirstPage(): Promise<void> {
  listed = [];
  final_page = true;
  await loadPage();
}

export async function loadPage(): Promise<void> {
  pending = true;
  error = null;

  try {
    const page = await roomsAPI.listDiscoverable(listed.at(-1)?.id);
    listed.push(...page);
    final_page = page.length < page_size;
  } catch (err) {
    error = failureText(err);
  } finally {
    pending = false;
  }
}

export async function join(room_id: string): Promise<void> {
  pending = true;
  error = null;

  try {
    await roomsAPI.join(room_id);
  } catch (err) {
    error = failureText(err);
  } finally {
    pending = false;
  }
}

events.subscribe(LocalEvent.server_info_loaded, (info: ServerInfo) => {
  page_size = info.limit_room_page;
});

events.subscribe(LocalEvent.rooms_changed, (room_ids: string[]) => {
  joined = new Set(room_ids);
});
