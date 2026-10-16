import type { ServerInfo } from "@/bindings/routes/server";
import * as serverAPI from "@/lib/api/server";
import { getUrl } from "@/lib/session.svelte";

let info: ServerInfo | null = $state(null);

export function getInfo(): ServerInfo | null {
  return info;
}

// Fetches what the active server reports about itself
export async function loadInfo(): Promise<void> {
  const url = getUrl();
  if (url === null) {
    return;
  }

  info = await serverAPI.getInfo(url);
}
