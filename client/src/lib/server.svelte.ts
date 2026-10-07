import type { ServerInfo } from "@/bindings/routes/server";
import { getServerInfo as fetchServerInfo } from "@/lib/api/server";
import { getUrl } from "@/lib/session.svelte";

let info: ServerInfo | null = $state(null);

export function getServerInfo(): ServerInfo | null {
  return info;
}

// Fetches what the active server reports about itself
export async function loadServerInfo(): Promise<void> {
  const url = getUrl();
  if (url === null) {
    return;
  }

  info = await fetchServerInfo(url);
}
