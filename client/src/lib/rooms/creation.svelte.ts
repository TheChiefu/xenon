import type { CreateRoomRequest } from "@/bindings/routes/rooms";
import { createRoom, getRoom } from "@/lib/api/rooms";
import { roomAdd } from "@/lib/rooms/layout.svelte";

// State //
let visible: boolean = $state(false);     // Dialog is on screen
let pending: boolean = $state(false);     // Sent, and the server has not answered
let error: string | null = $state(null);  // Message from the last failure

// Getters //

export function getError(): string | null {
  return error;
}

export function isOpen(): boolean {
  return visible;
}

export function isPending(): boolean {
  return pending;
}

// Dialog //

export function close(): void {
  visible = false;
}

export function open(): void {
  visible = true;
  pending = false;
  error = null;
}

// Server //

export async function submit(body: CreateRoomRequest): Promise<void> {
  pending = true;
  error = null;

  try {
    const created = await createRoom(body);

    // Create answers with an id alone
    roomAdd(await getRoom(created.id));
    visible = false;
  } catch (err) {
    error = err instanceof Error ? err.message : String(err);
  } finally {
    pending = false;
  }
}
