import type { ErrorResponse } from "@/bindings/error";
import { getToken, getUrl } from "@/lib/session.svelte";

// Types / Interfaces //

export interface FetchedFile {
    url: string;
    mime: string;
}

export type Method = "GET" | "POST" | "PATCH" | "DELETE" | "QUERY";


// Public Methods //

// Read a CSS time variable (e.g. "0.15s" or "150ms") from the page as milliseconds
export function cssTime(name: string): number {
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    const number = parseFloat(value);
    if (isNaN(number)) {
        return 0;
    }

    return value.endsWith("ms") ? number : number * 1000;
}

// Attaches the active session's address and bearer token, and throws the
// server's error message on a failing status
export async function request(path: string, method?: Method, body?: unknown): Promise<Response> {
    const url = getUrl();
    const token = getToken();

    if (url === null || token === null) {
        throw new Error("Not signed in");
    }

    const headers: Record<string, string> = { "Authorization": `Bearer ${token}` };
    let payload: string | FormData | undefined = undefined;

    if (body instanceof FormData) {
        payload = body;
    } else if (body !== undefined) {
        headers["Content-Type"] = "application/json";
        payload = JSON.stringify(body);
    }

    // An undefined method is a GET (fetch's default)
    const response = await fetch(`${url}${path}`, {
        method,
        headers,
        body: payload,
    });

    // Failure throws error message from server
    if (!response.ok) {
        throw new Error(await errorMessage(response));
    }

    return response;
}

// Fetch a file as a BLOB, interpret file from mime type
export async function fetchFileBlob(fileId: string): Promise<FetchedFile> {
    const response = await request(`/files/${fileId}`);

    const mime = response.headers.get("x-file-mime") ?? "application/octet-stream";
    const bytes = await response.arrayBuffer();
    const blob = new Blob([bytes], { type: mime });
    return { url: URL.createObjectURL(blob), mime };
}


// Helper Methods //

// The message the server sent, for a response already known to have failed
async function errorMessage(response: Response): Promise<string> {
    const text = await response.text();

    // Nothing in the body leaves the status as the only thing to report
    if (text === "") {
        return `Request failed with status ${response.status}`;
    }

    let body: unknown;
    try {
        body = JSON.parse(text);
    } catch {
        return text;
    }

    // Unknown body type, return response text as is
    if (typeof body !== "object" || body === null) {
        return text;
    }

    // Return response text as is if it does not match server error form
    if (Object.keys(body).length !== 1) {
        return text;
    }

    // Extract error message from server error response object
    const error = (body as Partial<ErrorResponse>).error;
    return typeof error === "string" ? error : text;
}
