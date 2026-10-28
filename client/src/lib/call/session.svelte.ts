import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { CallCloseCode, type CallServerEvent } from "@/bindings/sockets/call/events";
import { CallEventType, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
import * as profile from "@/lib/users/profile.svelte";
import * as levels from "@/lib/call/levels.svelte";
import * as peers from "@/lib/call/peers.svelte";
import * as signaling from "@/lib/call/signaling.svelte";
import * as socket from "@/lib/call/socket.svelte";
import { LABELS_CALL_CLOSE, MSG_MICROPHONE_DENIED } from "@/lib/labels";

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
  const volume = volumes.get(user_id);

  if (volume === undefined) {
    return VOLUME_MAX;
  }

  return volume;
}

export function isUserMuted(user_id: string): boolean {
  return muted_users.has(user_id);
}

// CALL //

// Joins a room's call
export async function join(roomId: string): Promise<void> {
  if (room_id !== null) {
    return;
  }

  failure = null;

  // Ask browser for microphone
  let microphone: MediaStream;

  try {
    microphone = await navigator.mediaDevices.getUserMedia({
      audio: { noiseSuppression: false, autoGainControl: false },
    });
  } catch {
    failure = MSG_MICROPHONE_DENIED;
    return;
  }

  // Apply toggles to new microphone and audio
  peers.setMicrophone(microphone);
  peers.setMicrophoneEnabled(mic_enabled);
  peers.setAudioMuted(!sound_enabled);

  // Measure own microphone for speaking indicator
  const self_id = profile.get()?.id;

  if (self_id !== undefined) {
    levels.watch(self_id, microphone);
  }

  room_id = roomId;
  socket.open(roomId, receive, closed);
}

// Leaves the call
export function leave(): void {
  socket.close();
  end();
}

// TOGGLES //

export function setMicEnabled(enabled: boolean): void {
  mic_enabled = enabled;
  peers.setMicrophoneEnabled(enabled);
}

export function setSoundEnabled(enabled: boolean): void {
  sound_enabled = enabled;
  peers.setAudioMuted(!enabled);
}

// Sets one participant's volume, as a percent
export function setVolume(user_id: string, percent: number): void {
  volumes.set(user_id, percent);
  peers.setVolume(user_id, percent / VOLUME_MAX);
}

// Mutes or unmutes one participant on this client
export function setUserMuted(user_id: string, muted: boolean): void {
  if (muted) {
    muted_users.add(user_id);
  } else {
    muted_users.delete(user_id);
  }

  peers.setUserMuted(user_id, muted);
}

// EVENTS //

// Handles an event from the call socket
function receive(event: CallServerEvent): void {
  switch (event.type) {
    case CallEventType.call_joined: {
      peers.setIceServers(event.ice_servers);

      const self_id = profile.get()?.id;

      // Send SDP offer to everyone already in call
      for (const user_id of event.participants) {
        if (user_id !== self_id) {
          signaling.queue(() => signaling.offer(user_id));
        }
      }
      break;
    }

    case CallEventType.signal:
      signaling.queue(() => signaling.receive(event.from, event.payload));
      break;
  }
}

// Ends the call after the call socket closes
function closed(code: number): void {

  // Show reason for a known close code
  switch (code) {
    case CallCloseCode.server_error:
    case CallCloseCode.no_connect:
    case CallCloseCode.already_in_call:
    case CallCloseCode.call_full:
      failure = LABELS_CALL_CLOSE[code];
      break;
  }

  end();
}

// Ends the call
function end(): void {
  signaling.reset();
  peers.clear();
  levels.clear();
  room_id = null;
}

// LISTENERS //

// Close the connection to a participant who left this call
events.subscribe(ServerEventType.participant_left, (event) => {
  if (event.room_id === room_id) {
    peers.remove(event.user_id);
  }
});
