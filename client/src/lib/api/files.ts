import type { FileResponse } from "@/bindings/routes/files";
import { request } from "@/lib/utils";

export async function uploadFile(file: File): Promise<FileResponse> {
    const form = new FormData();
    form.append("file", file);

    const response = await request("/files", "POST", form);
    return await response.json() as FileResponse;
}
