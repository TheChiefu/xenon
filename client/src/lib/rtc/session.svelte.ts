import { SvelteMap, SvelteSet } from "svelte/reactivity";
import type { RtcCloseCode } from "@/bindings/rtc/events";
import { ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
import * as connection from "@/lib/rtc/connection.svelte";
import * as levels from "@/lib/rtc/levels.svelte";
import * as profile from "@/lib/users/profile.svelte";
import { LABELS_RTC_CLOSE, MSG_MICROPHONE_DENIED } from "@/lib/labels";

// A screen share's stream id is the user id with this added
const SCREEN_SUFFIX: string = ":screen";

export const VOLUME_MAX: number = 100;

// STATE //

let room_id: string | null = $state(null);
let failure: string | null = $state(null);
let mic_enabled: boolean = $state(true);
let sound_enabled: boolean = $state(true);

// Volume percent of each participant, keyed by user id
const volumes: SvelteMap<string, number> = new SvelteMap();

// Participants muted on this client only
const muted_users: SvelteSet<string> = new SvelteSet();

// Playback of each participant's microphone, keyed by user id
const players: Map<string, HTMLAudioElement> = new Map();

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

  leave();

  connection.open(roomId, microphone, play, closed);
  connection.setMicrophoneEnabled(mic_enabled);

  // Measure own microphone for the speaking bar
  const self_id = profile.get()?.id;
  if (self_id !== undefined) {
    levels.watch(self_id, microphone);
  }

  room_id = roomId;
}

export function leave(): void {
  connection.close();

  for (const user_id of [...players.keys()]) {
    stop(user_id);
  }

  levels.clear();
  room_id = null;
}

// SETTINGS //

export function setMicEnabled(enabled: boolean): void {
  mic_enabled = enabled;
  connection.setMicrophoneEnabled(enabled);
}

export function setSoundEnabled(enabled: boolean): void {
  sound_enabled = enabled;
  players.forEach((audio, user_id) => applyMuted(user_id, audio));
}

// Sets one participant's volume, as a percent
export function setVolume(user_id: string, percent: number): void {
  volumes.set(user_id, percent);

  const audio = players.get(user_id);
  if (audio !== undefined) {
    audio.volume = percent / VOLUME_MAX;
  }
}

// Mutes or unmutes one participant on this client
export function setUserMuted(user_id: string, muted: boolean): void {
  if (muted) {
    muted_users.add(user_id);
  } else {
    muted_users.delete(user_id);
  }

  const audio = players.get(user_id);
  if (audio !== undefined) {
    applyMuted(user_id, audio);
  }
}

// HELPERS //

// Plays a participant's microphone. Camera and screen share are not shown.
function play(event: RTCTrackEvent): void {
  const stream = event.streams[0];

  if (stream === undefined || event.track.kind !== "audio" || stream.id.endsWith(SCREEN_SUFFIX)) {
    return;
  }

  const user_id = stream.id;
  stop(user_id);

  const audio = new Audio();
  audio.srcObject = stream;
  audio.volume = getVolume(user_id) / VOLUME_MAX;
  applyMuted(user_id, audio);

  players.set(user_id, audio);
  levels.watch(user_id, stream);

  // Playback started before any user interaction is blocked by the browser
  audio.play().catch(console.warn);
}

function stop(user_id: string): void {
  const audio = players.get(user_id);

  if (audio === undefined) {
    return;
  }

  audio.srcObject = null;
  players.delete(user_id);
  levels.unwatch(user_id);
}

function applyMuted(user_id: string, audio: HTMLAudioElement): void {
  audio.muted = !sound_enabled || muted_users.has(user_id);
}

// Ends the session after the RTC socket closes or the connection fails, showing why when known
function closed(code: number): void {
  failure = LABELS_RTC_CLOSE[code as RtcCloseCode] ?? null;
  leave();
}

// LISTENERS //

// Stop playing a participant who left this room
events.subscribe(ServerEventType.rtc_left, (event) => {
  if (event.room_id === room_id) {
    stop(event.user_id);
  }
});
