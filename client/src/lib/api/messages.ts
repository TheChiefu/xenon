import type { EditMessageRequest, EditMessageResponse, MessageResponse, PostMessageRequest } from "@/bindings/routes/messages";
import type { ServerInfo } from "@/bindings/routes/server";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import { request } from "@/lib/utils";

let max_attachments: number | null = null;
let page_size: number | null = null;

export function attachmentLimit(): number | null {
    return max_attachments;
}

export function pageSize(): number | null {
    return page_size;
}

export async function fetch(roomId: string, before?: number, after?: number): Promise<MessageResponse[]> {
    const query = new URLSearchParams();

    if (before !== undefined) {
        query.set("before", String(before));
    }

    if (after !== undefined) {
        query.set("after", String(after));
    }

    const response = await request(`/rooms/${roomId}/messages?${query}`);
    return await response.json() as MessageResponse[];
}

export async function get(messageId: string): Promise<MessageResponse> {
    const response = await request(`/messages/${messageId}`);
    return await response.json() as MessageResponse;
}

export async function post(roomId: string, body: PostMessageRequest): Promise<MessageResponse> {
    if (max_attachments !== null && body.attachments.length > max_attachments) {
        throw new Error(`A message can have at most ${max_attachments} attachments`);
    }

    const response = await request(`/rooms/${roomId}/messages`, "POST", body);
    return await response.json() as MessageResponse;
}

export async function edit(messageId: string, body: EditMessageRequest): Promise<EditMessageResponse> {
    const response = await request(`/messages/${messageId}`, "PATCH", body);
    return await response.json() as EditMessageResponse;
}

export async function remove(messageId: string): Promise<void> {
    await request(`/messages/${messageId}`, "DELETE");
}

// Events
events.subscribe(LocalEvent.server_info_loaded, (info: ServerInfo) => {
    max_attachments = info.limit_attachments_per_message_max;
    page_size = info.limit_message_page;
});
