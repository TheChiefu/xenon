import type { ServerInfo } from "@/bindings/routes/server";
import * as serverAPI from "@/lib/api/server";
import * as session from "@/lib/session.svelte";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

let info: ServerInfo | null = $state(null);

export function getInfo(): ServerInfo | null {
  return info;
}

// Fetches what the active server reports about itself
export async function loadInfo(): Promise<void> {
  const url = session.getUrl();
  if (url === null) {
    return;
  }

  info = await serverAPI.getInfo(url);
  events.dispatch(LocalEvent.server_info_loaded, info);
}
