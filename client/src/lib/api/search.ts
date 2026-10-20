import type { SearchResult } from "@/bindings/routes/search";
import type { ServerInfo } from "@/bindings/routes/server";
import type { SearchRequest } from "@/bindings/shared";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import { request } from "@/lib/utils";

let page_size: number | null = null;

export function pageSize(): number | null {
    return page_size;
}

export async function messages(body: SearchRequest): Promise<SearchResult[]> {
    const response = await request("/search", "POST", body);
    return await response.json() as SearchResult[];
}

// Events
events.subscribe(LocalEvent.server_info_loaded, (info: ServerInfo) => {
    page_size = info.limit_search_page;
});
