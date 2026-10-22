import * as peers from "@/lib/call/peers.svelte";
import * as socket from "@/lib/call/socket.svelte";
import type { Signal } from "@/lib/call/socket.svelte";

// STATE //

// SDP and ICE operations waiting to run, oldest first
let pending: Array<() => Promise<void>> = [];
let running: boolean = false;

// QUEUE //

// Queues an SDP or ICE operation
export function queue(operation: () => Promise<void>): void {
  pending.push(operation);
  run();
}

// Drops every queued operation
export function reset(): void {
  pending = [];
}

// Runs queued operations one at a time, as an ICE candidate needs its SDP description applied first
async function run(): Promise<void> {
  if (running) {
    return;
  }

  running = true;

  let operation = pending.shift();

  while (operation !== undefined) {
    try {
      await operation();
    } catch (error) {
      console.warn(error);
    }

    operation = pending.shift();
  }

  running = false;
}

// SIGNALS //

// Sends an SDP offer to a participant already in the call
export async function offer(user_id: string): Promise<void> {
  const peer = peers.create(user_id);
  const description = await peer.connection.createOffer();
  await peer.connection.setLocalDescription(description);

  socket.sendSignal(user_id, { description });
}

// Applies a signaling message from another participant
export async function receive(from: string, payload: string): Promise<void> {
  let signal: Signal;

  try {
    signal = JSON.parse(payload);
  } catch {
    return;
  }

  if (signal.description !== undefined) {
    await receiveDescription(from, signal.description);
  }

  if (signal.candidate !== undefined) {
    await receiveCandidate(from, signal.candidate);
  }
}

// HELPERS //

// Applies an SDP description from another participant
async function receiveDescription(
  from: string,
  description: RTCSessionDescriptionInit,
): Promise<void> {

  // Answer an SDP offer
  if (description.type === "offer") {
    const peer = peers.create(from);
    await peer.connection.setRemoteDescription(description);

    const answer = await peer.connection.createAnswer();
    await peer.connection.setLocalDescription(answer);

    socket.sendSignal(from, { description: answer });
    return;
  }

  // Apply an SDP answer to the offer sent
  const peer = peers.get(from);

  if (peer !== undefined) {
    await peer.connection.setRemoteDescription(description);
  }
}

// Adds an ICE candidate to the sender's connection
async function receiveCandidate(
  from: string,
  candidate: RTCIceCandidateInit,
): Promise<void> {
  const peer = peers.get(from);

  if (peer !== undefined) {
    await peer.connection.addIceCandidate(candidate);
  }
}
