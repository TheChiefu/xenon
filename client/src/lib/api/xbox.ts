import type { FriendResponse, LinkResponse, XboxPatch, XboxResponse } from "@/bindings/services/xbox";
import { request } from "@/lib/utils";

export async function getAccount(): Promise<XboxResponse | null> {
    const response = await request("/me/xbox");
    return await response.json() as XboxResponse | null;
}

export async function startLink(): Promise<LinkResponse> {
    const response = await request("/me/xbox", "POST");
    return await response.json() as LinkResponse;
}

export async function updateAccount(body: XboxPatch): Promise<void> {
    await request("/me/xbox", "PATCH", body);
}

export async function unlink(): Promise<void> {
    await request("/me/xbox", "DELETE");
}

export async function getFriends(): Promise<FriendResponse[]> {
    const response = await request("/me/xbox/friends");
    return await response.json() as FriendResponse[];
}
