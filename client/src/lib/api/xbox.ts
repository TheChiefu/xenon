import type { LinkResponse } from "@/bindings/services/xbox";
import { request } from "@/lib/utils";

export async function startXboxLink(): Promise<LinkResponse> {
    const response = await request("/me/xbox", "POST");
    return await response.json() as LinkResponse;
}

export async function unlinkXbox(): Promise<void> {
    await request("/me/xbox", "DELETE");
}
