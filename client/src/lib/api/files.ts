import type { FileResponse } from "@/bindings/routes/files";
import type { ServerInfo } from "@/bindings/routes/server";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import { formatBytes, request } from "@/lib/utils";

let max_bytes: number | null = null;

export async function upload(file: File): Promise<FileResponse> {

    // Reject a file over the server's limit before sending it
    if (max_bytes !== null && file.size > max_bytes) {
        throw new Error(`${file.name} is larger than ${formatBytes(max_bytes)}`);
    }

    const form = new FormData();
    form.append("file", file);

    const response = await request("/files", "POST", form);
    return await response.json() as FileResponse;
}

// Generate a new random file name with optional prefix 
export function generateName(file: File, prefix=""): File {
    const rand = crypto.randomUUID();
    const name = prefix ? `${prefix}_${rand}` : rand;
    const ext = file.type.split('/').at(-1);

    return new File(
        [file], `${name}.${ext}`, {
            type: file.type,
            lastModified: file.lastModified
        }
    )
}

events.subscribe(LocalEvent.server_info_loaded, (info: ServerInfo) => {
    max_bytes = info.limit_file_bytes_max;
});
