import type { FriendResponse, LinkResponse, XboxPatch, XboxResponse } from "@/bindings/services/xbox";
import { request } from "@/lib/utils";

export async function getXboxAccount(): Promise<XboxResponse | null> {
    const response = await request("/me/xbox");
    return await response.json() as XboxResponse | null;
}

export async function startXboxLink(): Promise<LinkResponse> {
    const response = await request("/me/xbox", "POST");
    return await response.json() as LinkResponse;
}

export async function updateXboxAccount(body: XboxPatch): Promise<void> {
    await request("/me/xbox", "PATCH", body);
}

export async function unlinkXbox(): Promise<void> {
    await request("/me/xbox", "DELETE");
}

export async function getXboxFriends(): Promise<FriendResponse[]> {
    const response = await request("/me/xbox/friends");
    return await response.json() as FriendResponse[];
}
