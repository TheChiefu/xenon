// The other participants' microphones, and how loud each plays on this client

import { SvelteMap, SvelteSet } from "svelte/reactivity";
import * as levels from "@/lib/rtc/levels.svelte";

export const VOLUME_MAX: number = 100;

// STATE //

let sound_enabled: boolean = $state(true);

// Volume percent of each participant, keyed by user id
const volumes: SvelteMap<string, number> = new SvelteMap();

// Participants muted on this client only
const muted_users: SvelteSet<string> = new SvelteSet();

// Playback of each participant's microphone, keyed by user id
const players: Map<string, HTMLAudioElement> = new Map();

// GETTERS //

export function getSoundEnabled(): boolean {
  return sound_enabled;
}

export function getVolume(user_id: string): number {
  return volumes.get(user_id) ?? VOLUME_MAX;
}

export function isUserMuted(user_id: string): boolean {
  return muted_users.has(user_id);
}

// SETTINGS //

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

// PLAYBACK //

// Plays a participant's microphone, replacing any playing already
export function play(user_id: string, stream: MediaStream): void {
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

export function stop(user_id: string): void {
  const audio = players.get(user_id);

  if (audio === undefined) {
    return;
  }

  audio.srcObject = null;
  players.delete(user_id);
  levels.unwatch(user_id);
}

export function clear(): void {
  for (const user_id of [...players.keys()]) {
    stop(user_id);
  }
}

// HELPERS //

function applyMuted(user_id: string, audio: HTMLAudioElement): void {
  audio.muted = !sound_enabled || muted_users.has(user_id);
}
