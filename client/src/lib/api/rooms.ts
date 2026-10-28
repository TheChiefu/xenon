import type { RespondRoomInvite, RespondRoomReadState } from "@/bindings/routes/me";
import type { Patch, PatchAccess, RequestCreateRoom, RequestRead, RespondAccess, RespondCreated, RespondDiscoverability, RespondMember } from "@/bindings/routes/rooms";
import type { RoomInfo } from "@/bindings/types/rooms";
import { request } from "@/lib/utils";

export async function create(body: RequestCreateRoom): Promise<RespondCreated> {
    const response = await request("/rooms", "POST", body);
    return await response.json() as RespondCreated;
}

export async function get(roomId: string): Promise<RoomInfo> {
    const response = await request(`/rooms/${roomId}`);
    return await response.json() as RoomInfo;
}

export async function listMembers(roomId: string): Promise<RespondMember[]> {
    const response = await request(`/rooms/${roomId}/members`);
    return await response.json() as RespondMember[];
}

export async function listMyRooms(): Promise<RoomInfo[]> {
    const response = await request("/me/rooms");
    return await response.json() as RoomInfo[];
}

export async function listMyReads(): Promise<RespondRoomReadState[]> {
    const response = await request("/me/read");
    return await response.json() as RespondRoomReadState[];
}

export async function markRead(room_id: string, last_read_seq: number, keepalive: boolean): Promise<void> {
    const body: RequestRead = { last_read_seq };
    await request(`/rooms/${room_id}/read`, "PUT", body, keepalive);
}

export async function listDiscoverable(after?: string): Promise<RespondDiscoverability[]> {
    const query = new URLSearchParams();

    if (after !== undefined) {
        query.set("after", after);
    }

    const response = await request(`/rooms?${query}`);
    return await response.json() as RespondDiscoverability[];
}

export async function join(roomId: string): Promise<void> {
    await request(`/rooms/${roomId}/join`, "POST");
}

export async function leave(room_id: string): Promise<void> {
    await request(`/rooms/${room_id}/leave`, "POST");
}

export async function update(room_id: string, body: Patch): Promise<void> {
    await request(`/rooms/${room_id}`, "PATCH", body);
}

export async function remove(room_id: string): Promise<void> {
    await request(`/rooms/${room_id}`, "DELETE");
}

export async function getMember(room_id: string, user_id: string): Promise<RespondAccess> {
    const response = await request(`/rooms/${room_id}/members/${user_id}`);
    return await response.json() as RespondAccess;
}

export async function updateMember(room_id: string, user_id: string, body: PatchAccess): Promise<void> {
    await request(`/rooms/${room_id}/members/${user_id}`, "PATCH", body);
}

export async function listInvites(room_id: string): Promise<string[]> {
    const response = await request(`/rooms/${room_id}/invites`);
    return await response.json() as string[];
}

export async function invite(room_id: string, user_id: string): Promise<void> {
    await request(`/rooms/${room_id}/invites/${user_id}`, "PUT");
}

export async function revokeInvite(room_id: string, user_id: string): Promise<void> {
    await request(`/rooms/${room_id}/invites/${user_id}`, "DELETE");
}

export async function listMyInvites(): Promise<RespondRoomInvite[]> {
    const response = await request("/me/invites");
    return await response.json() as RespondRoomInvite[];
}

export async function declineInvite(room_id: string): Promise<void> {
    await request(`/me/invites/${room_id}`, "DELETE");
}

export async function listBans(room_id: string): Promise<string[]> {
    const response = await request(`/rooms/${room_id}/bans`);
    return await response.json() as string[];
}

export async function ban(room_id: string, user_id: string): Promise<void> {
    await request(`/rooms/${room_id}/bans/${user_id}`, "PUT");
}

export async function unban(room_id: string, user_id: string): Promise<void> {
    await request(`/rooms/${room_id}/bans/${user_id}`, "DELETE");
}
