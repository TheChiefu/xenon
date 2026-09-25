export interface FetchedFile {
    url: string;
    mime: string;
}

// Read a CSS time variable (e.g. "0.15s" or "150ms") from the page as milliseconds
export function cssTime(name: string): number {
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    const number = parseFloat(value);
    if (isNaN(number)) {
        return 0;
    }

    return value.endsWith("ms") ? number : number * 1000;
}

// Fetch a file as a BLOB, interpret file from mime type
export async function fetchFileBlob(url: string, fileId: string, token: string): Promise<FetchedFile> {
    const response = await fetch(`${url}/files/${fileId}`, {
        headers: { "Authorization": `Bearer ${token}` },
    });

    if (!response.ok) {
        throw new Error("Failed to fetch file");
    }

    const mime = response.headers.get("x-file-mime") ?? "application/octet-stream";
    const bytes = await response.arrayBuffer();
    const blob = new Blob([bytes], { type: mime });
    return { url: URL.createObjectURL(blob), mime };
}
