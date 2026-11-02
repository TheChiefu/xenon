import * as levels from "@/lib/rtc/levels.svelte";

// Playback of each participant's microphone, keyed by user id
const players: Map<string, HTMLAudioElement> = new Map();

// Volume of each participant, from 0 to 1, keyed by user id
const volumes: Map<string, number> = new Map();

// Participants muted on this client only
const muted_users: Set<string> = new Set();

let all_muted: boolean = false;

// PLAYBACK //

// Plays a participant's microphone, replacing any earlier playback
export function play(user_id: string, stream: MediaStream): void {
  stop(user_id);

  const audio = new Audio();
  audio.srcObject = stream;
  audio.volume = volumes.get(user_id) ?? 1;
  audio.muted = all_muted || muted_users.has(user_id);

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

  levels.clear();
}

// SETTINGS //

// Sets a participant's volume, from 0 to 1
export function setVolume(user_id: string, volume: number): void {
  volumes.set(user_id, volume);

  const audio = players.get(user_id);

  if (audio !== undefined) {
    audio.volume = volume;
  }
}

export function setUserMuted(user_id: string, muted: boolean): void {
  if (muted) {
    muted_users.add(user_id);
  } else {
    muted_users.delete(user_id);
  }

  const audio = players.get(user_id);

  if (audio !== undefined) {
    audio.muted = all_muted || muted;
  }
}

export function setAllMuted(muted: boolean): void {
  all_muted = muted;

  for (const [user_id, audio] of players) {
    audio.muted = muted || muted_users.has(user_id);
  }
}
