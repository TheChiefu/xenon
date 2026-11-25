import type { LinkResponse } from "@/bindings/services/steam";
import { request } from "@/lib/utils";

export async function startLink(): Promise<LinkResponse> {
    const response = await request("/me/steam", "POST");
    return await response.json() as LinkResponse;
}

export async function unlink(): Promise<void> {
    await request("/me/steam", "DELETE");
}
