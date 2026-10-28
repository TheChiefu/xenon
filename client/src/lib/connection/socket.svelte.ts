import type { Status } from "@/bindings/shared";
import type { ClientEvent, ServerEvent } from "@/bindings/sockets/session/events";
import { Device } from "@/bindings/sockets/session/presence";
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import * as session from "@/lib/connection/session.svelte";
import * as status from "@/lib/users/status.svelte";

let socket: WebSocket | null = null;
let connected: boolean = $state(false);

// CONNECTION //

export function isConnected(): boolean {
  return connected;
}

export function connect(): void {
  if (socket !== null) {
    return;
  }

  const url = session.getUrl();
  const token = session.getToken();

  if (url === null || token === null) {
    return;
  }

  const address = new URL(`${url}/ws`);
  address.protocol = address.protocol === "https:" ? "wss:" : "ws:";
  address.searchParams.set("status", status.get());

  const device = deviceName();
  if (device !== null) {
    address.searchParams.set("device", device);
  }

  const opened = new WebSocket(address, ["Bearer", token]);
  socket = opened;

  opened.onopen = () => {
    connected = true;
  };

  opened.onclose = () => {
    connected = false;
    socket = null;
  };

  opened.onmessage = (message: MessageEvent) => {
    if (typeof message.data === "string") {
      receive(message.data);
    }
  };
}

export function close(): void {
  if (socket === null) {
    return;
  }

  const closing = socket;
  socket = null;
  connected = false;
  closing.onclose = null;
  closing.close();
}

export function send(event: ClientEvent): void {
  if (socket === null || socket.readyState !== WebSocket.OPEN) {
    return;
  }

  socket.send(JSON.stringify(event));
}

// EVENTS //

function receive(data: string): void {
  let received: ServerEvent;

  try {
    received = JSON.parse(data);
  } catch {
    return;
  }

  events.dispatch(received.type, received);
}

// Send user's new status to the server
events.subscribe(LocalEvent.status_changed, (status: Status) => {
  send({ type: "status", status });
});

// HELPERS //

function deviceName(): Device | null {
  const agent = navigator.userAgent;

  if (/Android/i.test(agent)) {
    return Device.android;
  }

  if (/iPhone|iPad|iPod/i.test(agent)) {
    return Device.ios;
  }

  if (/Windows/i.test(agent)) {
    return Device.windows;
  }

  if (/Macintosh|Mac OS X/i.test(agent)) {
    return Device.macos;
  }

  if (/Linux|X11|CrOS/i.test(agent)) {
    return Device.linux;
  }

  return null;
}
