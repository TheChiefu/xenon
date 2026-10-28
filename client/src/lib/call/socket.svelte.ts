import type { CallClientEvent, CallServerEvent } from "@/bindings/sockets/call/events";
import { CallEventType } from "@/lib/events";
import * as session from "@/lib/connection/session.svelte";

// Signaling message: an SDP description or an ICE candidate
export interface Signal {
  description?: RTCSessionDescriptionInit;
  candidate?: RTCIceCandidateInit;
}

let socket: WebSocket | null = null;

// CONNECTION //

// Opens the call socket for a room, which joins its call
export function open(
  roomId: string,
  onEvent: (event: CallServerEvent) => void,
  onClose: (code: number) => void,
): void {
  if (socket !== null) {
    return;
  }

  const url = session.getUrl();
  const token = session.getToken();

  if (url === null || token === null) {
    return;
  }

  const address = new URL(`${url}/rooms/${roomId}/call`);
  address.protocol = address.protocol === "https:" ? "wss:" : "ws:";

  // Authenticates with the session token
  const opened = new WebSocket(address, ["Bearer", token]);
  socket = opened;

  // Reports a close the client did not start:
  // - a refused join
  // - the call ending with its room
  // - a dropped connection
  opened.onclose = (closed: CloseEvent) => {
    socket = null;
    onClose(closed.code);
  };

  // Passes each event the server sends on, skipping any that do not parse
  opened.onmessage = (message: MessageEvent) => {
    if (typeof message.data !== "string") {
      return;
    }

    let received: CallServerEvent;

    try {
      received = JSON.parse(message.data);
    } catch {
      return;
    }

    onEvent(received);
  };
}

// Closes the call
export function close(): void {
  if (socket === null) {
    return;
  }

  const closing = socket;
  socket = null;

  // Detach close handler
  closing.onclose = null;

  // Close socket (leave call)
  closing.close();
}

// Sends an event to the server, dropping it while the socket is not open
export function send(event: CallClientEvent): void {
  if (socket === null || socket.readyState !== WebSocket.OPEN) {
    return;
  }

  socket.send(JSON.stringify(event));
}

// Sends a signal to another participant through the server
export function sendSignal(to: string, signal: Signal): void {
  send({
    type: CallEventType.signal,
    to,
    payload: JSON.stringify(signal),
  });
}
