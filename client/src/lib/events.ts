// Listeners are registered under an event name,
// publishing the name calls each of its listeners, passing them the published data

import type { CallServerEvent } from "@/bindings/sockets/call/events";
import type { ServerEvent } from "@/bindings/sockets/session/events";
import { CHOICE_NO, CHOICE_YES } from "@/lib/labels";

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
  appearance_updated = "appearance_updated",
  appearance_snapshot = "appearance_snapshot",
  platform_snapshot = "platform_snapshot",
  platform_updated = "platform_updated",
  profile_updated = "profile_updated",
  resync = "resync",
  notification = "notification",
  call_snapshot = "call_snapshot",
  participant_joined = "participant_joined",
  participant_left = "participant_left",
  session_revoked = "session_revoked",
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
  appearance_changed = "appearance_changed",
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

// A question with one button per choice, answered with the clicked button's position, or null when dismissed
export interface ConfirmRequest {
  message: string;
  choices: string[];
  answer: (choice: number | null) => void;
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

// Asks the user a question, resolving to the clicked button's position in choices, or null when dismissed
export function choose(message: string, choices: string[]): Promise<number | null> {
  return new Promise<number | null>((resolve) => {
    const request: ConfirmRequest = { message, choices, answer: resolve };
    dispatch(LocalEvent.confirm_requested, request);
  });
}

// Asks the user a Yes/No question, resolving to their answer
export async function confirm(message: string): Promise<boolean> {
  const choice = await choose(message, [CHOICE_YES, CHOICE_NO]);
  return choice === 0;
}

export function subscribe(type: EventType, listener: (data: any) => void): void {
  events.addEventListener(type, (event) => {
    const published = event as CustomEvent;
    listener(published.detail);
  });
}
