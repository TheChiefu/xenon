import { SvelteMap } from "svelte/reactivity";
import { fetchFileBlob, type FetchedFile } from "@/lib/utils";

const files: SvelteMap<string, FetchedFile> = new SvelteMap();

// Gets a file already fetched from the server
export function getFile(fileId: string): FetchedFile | undefined {
  return files.get(fileId);
}

// Fetches file(s) that have not been fetched before
export async function loadFiles(fileIds: string[]): Promise<void> {
  const missing: string[] = [...new Set(fileIds)].filter((id) => !files.has(id));

  await Promise.all(missing.map(async (id) => {
    files.set(id, await fetchFileBlob(id));
  }));
}
