import type { RtcClientEvent, RtcServerEvent } from "@/bindings/rtc/events";
import * as session from "@/lib/connection/session.svelte";

let socket: WebSocket | null = null;

// CONNECTION //

// Opens the RTC socket for a room, which joins its connections
export function open(
  roomId: string,
  onEvent: (event: RtcServerEvent) => void,
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

  const address = new URL(`${url}/rooms/${roomId}/rtc`);
  address.protocol = address.protocol === "https:" ? "wss:" : "ws:";

  const opened = new WebSocket(address, ["Bearer", token]);
  socket = opened;

  opened.onclose = (closed: CloseEvent) => {
    socket = null;
    onClose(closed.code);
  };

  opened.onmessage = (message: MessageEvent) => {
    if (typeof message.data !== "string") {
      return;
    }

    let received: RtcServerEvent;

    try {
      received = JSON.parse(message.data);
    } catch {
      return;
    }

    onEvent(received);
  };
}

// Closes the RTC socket without reporting the close
export function close(): void {
  if (socket === null) {
    return;
  }

  const closing = socket;
  socket = null;
  closing.onclose = null;
  closing.close();
}

export function send(event: RtcClientEvent): void {
  if (socket === null || socket.readyState !== WebSocket.OPEN) {
    return;
  }

  socket.send(JSON.stringify(event));
}
