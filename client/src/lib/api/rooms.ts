import type { CreateRoomRequest, CreateRoomResponse, MemberResponse, RoomResponse } from "@/bindings/routes/rooms";
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
