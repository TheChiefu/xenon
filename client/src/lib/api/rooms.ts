import type { BanResponse, CreateRoomRequest, CreateRoomResponse, DiscoverableRoomResponse, IssuedInviteResponse, MemberResponse, ReceivedInviteResponse, RoomAccessPatch, RoomPatch, RoomResponse } from "@/bindings/routes/rooms";
import type { ReadRequest, ReadResponse } from "@/bindings/routes/rooms";
import { request } from "@/lib/utils";

export async function create(body: CreateRoomRequest): Promise<CreateRoomResponse> {
    const response = await request("/rooms", "POST", body);
    return await response.json() as CreateRoomResponse;
}

export async function get(roomId: string): Promise<RoomResponse> {
    const response = await request(`/rooms/${roomId}`);
    return await response.json() as RoomResponse;
}

export async function listMembers(roomId: string): Promise<MemberResponse[]> {
    const response = await request(`/rooms/${roomId}/members`);
    return await response.json() as MemberResponse[];
}

export async function listMyRooms(): Promise<RoomResponse[]> {
    const response = await request("/me/rooms");
    return await response.json() as RoomResponse[];
}

export async function listMyReads(): Promise<ReadResponse[]> {
    const response = await request("/me/read");
    return await response.json() as ReadResponse[];
}

export async function markRead(room_id: string, last_read_seq: number, keepalive: boolean): Promise<void> {
    const body: ReadRequest = { last_read_seq };
    await request(`/rooms/${room_id}/read`, "PUT", body, keepalive);
}

export async function listDiscoverable(after?: string): Promise<DiscoverableRoomResponse[]> {
    const query = new URLSearchParams();

    if (after !== undefined) {
        query.set("after", after);
    }

    const response = await request(`/rooms?${query}`);
    return await response.json() as DiscoverableRoomResponse[];
}

export async function join(roomId: string): Promise<void> {
    await request(`/rooms/${roomId}/join`, "POST");
}

export async function leave(room_id: string): Promise<void> {
    await request(`/rooms/${room_id}/leave`, "POST");
}

export async function update(room_id: string, body: RoomPatch): Promise<void> {
    await request(`/rooms/${room_id}`, "PATCH", body);
}

export async function remove(room_id: string): Promise<void> {
    await request(`/rooms/${room_id}`, "DELETE");
}

export async function updateMember(room_id: string, user_id: string, body: RoomAccessPatch): Promise<void> {
    await request(`/rooms/${room_id}/members/${user_id}`, "PATCH", body);
}

export async function listInvites(room_id: string): Promise<IssuedInviteResponse[]> {
    const response = await request(`/rooms/${room_id}/invites`);
    return await response.json() as IssuedInviteResponse[];
}

export async function invite(room_id: string, invitee: string, expire_delta: number | null): Promise<void> {
    await request(`/rooms/${room_id}/invites`, "POST", { invitee, expire_delta });
}

export async function revokeInvite(room_id: string, user_id: string): Promise<void> {
    await request(`/rooms/${room_id}/invites/${user_id}`, "DELETE");
}

export async function listMyInvites(): Promise<ReceivedInviteResponse[]> {
    const response = await request("/me/invites");
    return await response.json() as ReceivedInviteResponse[];
}

export async function declineInvite(room_id: string): Promise<void> {
    await request(`/me/invites/${room_id}`, "DELETE");
}

export async function listBans(room_id: string): Promise<BanResponse[]> {
    const response = await request(`/rooms/${room_id}/bans`);
    return await response.json() as BanResponse[];
}

export async function unban(room_id: string, user_id: string): Promise<void> {
    await request(`/rooms/${room_id}/bans/${user_id}`, "DELETE");
}
