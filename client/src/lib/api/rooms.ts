import type { RoomResponse } from "@/bindings/routes/rooms";

export async function listMyRooms(url: string, token: string): Promise<RoomResponse[]> {
    const response = await fetch(`${url}/me/rooms`, {
        headers: { "Authorization": `Bearer ${token}` },
    });

    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return data as RoomResponse[];
}
