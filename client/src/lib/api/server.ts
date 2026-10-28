import type { ServerInfo } from "@/bindings/routes/server";

export async function getInfo(url: string): Promise<ServerInfo> {
    const response = await fetch(`${url}/server`);
    const data = await response.json();

    if (!response.ok) {
        throw new Error(data.error);
    }

    return data as ServerInfo;
}
