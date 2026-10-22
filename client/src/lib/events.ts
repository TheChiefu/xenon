// Listeners are registered under an event name,
// publishing the name calls each of its listeners, passing them the published data

import type { CallServerEvent, ServerEvent } from "@/bindings/sockets/events";

// TYPES //

// Names of events sent by the server
export enum ServerEventType {
  message = "message",
  message_deleted = "message_deleted",
  message_edited = "message_edited",
  invited = "invited",
  invite_revoked = "invite_revoked",
  banned = "banned",
  member_joined = "member_joined",
  member_left = "member_left",
  room_deleted = "room_deleted",
  room_updated = "room_updated",
  presence_updated = "presence_updated",
  presence_snapshot = "presence_snapshot",
  game_presence_snapshot = "game_presence_snapshot",
  game_presence_updated = "game_presence_updated",
  profile_updated = "profile_updated",
  resync = "resync",
  notification = "notification",
  call_snapshot = "call_snapshot",
  participant_joined = "participant_joined",
  participant_left = "participant_left",
}

// Check the enum has a matching entry for each server event type in the bindings
ServerEventType satisfies { [Type in ServerEvent["type"]]: Type };

// Names of events sent over the call socket
export enum CallEventType {
  call_joined = "call_joined",
  signal = "signal",
}

// Check the enum has a matching entry for each call socket event type in the bindings
CallEventType satisfies { [Type in CallServerEvent["type"]]: Type };

// Names of events sent within the client
export enum LocalEvent {
  signed_out = "signed_out",
  status_changed = "status_changed",
  user_selected = "user_selected",
  image_selected = "image_selected",
  confirm_requested = "confirm_requested",
  discover_requested = "discover_requested",
  settings_requested = "settings_requested",
  admin_requested = "admin_requested",
  server_info_loaded = "server_info_loaded",
  rooms_changed = "rooms_changed",
  room_settings_requested = "room_settings_requested",
  search_requested = "search_requested",
}

// A Yes/No question, answered by calling answer with the user's choice
export interface ConfirmRequest {
  message: string;
  answer: (result: boolean) => void;
}

// Event names accepted when publishing or listening
export type EventType = ServerEvent["type"] | LocalEvent;

// DOM event target
const events: EventTarget = new EventTarget();

// FUNCTIONS //

export function dispatch(type: EventType, data?: unknown): void {
  const event = new CustomEvent(type, { detail: data });
  events.dispatchEvent(event);
}

// Asks the user a Yes/No question, resolving to their answer
export function confirm(message: string): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    const request: ConfirmRequest = { message, answer: resolve };
    dispatch(LocalEvent.confirm_requested, request);
  });
}

export function subscribe(type: EventType, listener: (data: any) => void): void {
  events.addEventListener(type, (event) => {
    const published = event as CustomEvent;
    listener(published.detail);
  });
}
