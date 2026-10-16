import { Device, type ClientEvent, type ServerEvent } from "@/bindings/sockets/events";
import * as layout from "@/lib/rooms/layout.svelte";
import * as members from "@/lib/rooms/members.svelte";
import * as messages from "@/lib/messages.svelte";
import * as session from "@/lib/session.svelte";
import * as status from "@/lib/status.svelte";
import * as users from "@/lib/users.svelte";

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
  let event: ServerEvent;

  try {
    event = JSON.parse(data);
  } catch {
    return;
  }

  // Needs to be manually updated right now (see if we can use a proper generated enum)
  if (event.type === "presence_snapshot") {
    users.clearPresences();

    for (const user of event.users) {
      users.setPresence(user.user_id, user.status, user.device);
    }
    return;
  }

  if (event.type === "presence_updated") {
    users.setPresence(event.user_id, event.status, event.device);
    return;
  }

  if (event.type === "game_presence_snapshot") {
    users.clearActivities();

    for (const user of event.users) {
      users.setActivity(user.user_id, {
        platform: user.platform,
        status: user.status,
        title: user.title,
        detail: user.detail,
      });
    }
    return;
  }

  if (event.type === "game_presence_updated") {
    users.setActivity(event.user_id, {
      platform: event.platform,
      status: event.status,
      title: event.title,
      detail: event.detail,
    });
    return;
  }

  if (event.type === "profile_updated") {
    const summary = users.get(event.user_id);
    if (summary === undefined) {
      return;
    }

    users.set({
      id: event.user_id,
      username: summary.username,
      display_name: event.display_name,
      avatar_file_id: event.avatar_file_id,
      banner_file_id: event.banner_file_id,
      global_role: summary.global_role,
    });
    return;
  }

  if (event.type === "message") {
    messages.add(event.message);
    return;
  }

  if (event.type === "message_edited") {
    messages.edit(event.room_id, event.message_id, event.body, event.edited_at);
    return;
  }

  if (event.type === "message_deleted") {
    messages.remove(event.room_id, event.message_id);
    return;
  }

  if (event.type === "member_joined") {
    members.add(event.room_id, event.user_id);
  }

  if (event.type === "member_left") {
    members.remove(event.room_id, event.user_id);
  }

  if (event.type === "room_deleted") {
    members.leave(event.room_id);
  }

  if (event.type === "resync") {
    messages.clear();
    members.clear();
  }

  if (
    event.type === "invited"
    || event.type === "invite_revoked"
    || event.type === "banned"
    || event.type === "member_joined"
    || event.type === "member_left"
    || event.type === "room_updated"
    || event.type === "room_deleted"
    || event.type === "resync"
  ) {
    layout.loadRooms();
  }
}

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
