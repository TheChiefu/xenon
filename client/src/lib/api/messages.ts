import type { EditMessageRequest, EditMessageResponse, MessageResponse, PostMessageRequest } from "@/bindings/routes/messages";
import { request } from "@/lib/utils";

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
