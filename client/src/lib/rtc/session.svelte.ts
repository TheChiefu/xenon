import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { RtcCloseCode, RtcServerEvent } from "@/bindings/rtc/events";
import { RtcEventType, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
import * as audio from "@/lib/rtc/audio.svelte";
import * as connection from "@/lib/rtc/connection.svelte";
import * as levels from "@/lib/rtc/levels.svelte";
import * as socket from "@/lib/rtc/socket.svelte";
import * as profile from "@/lib/users/profile.svelte";
import { LABELS_RTC_CLOSE, MSG_MICROPHONE_DENIED } from "@/lib/labels";

// A screen share's stream id is the user id with this added
const SCREEN_SUFFIX: string = ":screen";

// STATE //

let room_id: string | null = $state(null);
let failure: string | null = $state(null);
let mic_enabled: boolean = $state(true);
let sound_enabled: boolean = $state(true);

// Volume percent of each participant, keyed by user id
const volumes: SvelteMap<string, number> = new SvelteMap();

// Participants muted on this client only
const muted_users: SvelteSet<string> = new SvelteSet();

export const VOLUME_MAX: number = 100;

// GETTERS //

export function getRoomId(): string | null {
  return room_id;
}

export function getFailure(): string | null {
  return failure;
}

export function getMicEnabled(): boolean {
  return mic_enabled;
}

export function getSoundEnabled(): boolean {
  return sound_enabled;
}

export function getVolume(user_id: string): number {
  return volumes.get(user_id) ?? VOLUME_MAX;
}

export function isUserMuted(user_id: string): boolean {
  return muted_users.has(user_id);
}

// JOINING //

// Joins a room's RTC connections, moving out of any other room's
export async function join(roomId: string): Promise<void> {
  if (room_id === roomId) {
    return;
  }

  failure = null;

  let microphone: MediaStream;

  try {
    microphone = await navigator.mediaDevices.getUserMedia({ audio: true });
  } catch {
    failure = MSG_MICROPHONE_DENIED;
    return;
  }

  end();

  connection.open(microphone, receiveTrack, end);
  connection.setMicrophoneEnabled(mic_enabled);
  audio.setAllMuted(!sound_enabled);

  // Measure own microphone for the speaking bar
  const self_id = profile.get()?.id;
  if (self_id !== undefined) {
    levels.watch(self_id, microphone);
  }

  room_id = roomId;
  socket.open(roomId, receive, closed);
}

export function leave(): void {
  socket.close();
  end();
}

// TOGGLES //

export function setMicEnabled(enabled: boolean): void {
  mic_enabled = enabled;
  connection.setMicrophoneEnabled(enabled);
}

export function setSoundEnabled(enabled: boolean): void {
  sound_enabled = enabled;
  audio.setAllMuted(!enabled);
}

// Sets one participant's volume, as a percent
export function setVolume(user_id: string, percent: number): void {
  volumes.set(user_id, percent);
  audio.setVolume(user_id, percent / VOLUME_MAX);
}

// Mutes or unmutes one participant on this client
export function setUserMuted(user_id: string, muted: boolean): void {
  if (muted) {
    muted_users.add(user_id);
  } else {
    muted_users.delete(user_id);
  }

  audio.setUserMuted(user_id, muted);
}

// EVENTS //

function receive(event: RtcServerEvent): void {
  switch (event.type) {
    case RtcEventType.offer:
      connection.receiveOffer(event.description).catch(console.warn);
      break;

    case RtcEventType.candidate:
      connection.receiveCandidate(event.candidate).catch(console.warn);
      break;
  }
}

// Plays a participant's microphone. Camera and screen share are not shown.
function receiveTrack(event: RTCTrackEvent): void {
  const stream = event.streams[0];

  if (stream === undefined || event.track.kind !== "audio" || stream.id.endsWith(SCREEN_SUFFIX)) {
    return;
  }

  audio.play(stream.id, stream);
}

// Ends the session after the RTC socket closes, showing why when known
function closed(code: number): void {
  failure = LABELS_RTC_CLOSE[code as RtcCloseCode] ?? null;
  end();
}

function end(): void {
  connection.close();
  audio.clear();
  room_id = null;
}

// LISTENERS //

// Stop playing a participant who left this room
events.subscribe(ServerEventType.rtc_left, (event) => {
  if (event.room_id === room_id) {
    audio.stop(event.user_id);
  }
});
