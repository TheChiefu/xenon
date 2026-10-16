import type { ServerInfo } from "@/bindings/routes/server";

export async function getInfo(url: string): Promise<ServerInfo> {
    const response = await fetch(`${url}/server`);
    return response.json();
}
