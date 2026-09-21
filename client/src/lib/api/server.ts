import type { ServerInfo } from "@/bindings/routes/server";

export async function getServerInfo(url: string): Promise<ServerInfo> {
    const response = await fetch(`${url}/server`);
    return response.json();
}
