import { SvelteMap } from "svelte/reactivity";
import * as levels from "@/lib/call/levels.svelte";
import * as socket from "@/lib/call/socket.svelte";

// Connection to one other participant, and its audio output
export interface Peer {
  connection: RTCPeerConnection;
  audio: HTMLAudioElement;
}

// STATE //

let microphone: MediaStream | null = null;
let ice_servers: RTCIceServer[] = [];
let audio_muted: boolean = false;
const peers: SvelteMap<string, Peer> = new SvelteMap();

// Playback volume of each participant, from 0 to 1, keyed by user id
const volumes: Map<string, number> = new Map();

// Participants muted on this client only
const muted_users: Set<string> = new Set();

// GETTERS //

export function get(user_id: string): Peer | undefined {
  return peers.get(user_id);
}

export function getUserIds(): string[] {
  return [...peers.keys()];
}

// MEDIA //

// Sets the microphone stream sent to every participant
export function setMicrophone(stream: MediaStream): void {
  microphone = stream;
}

// Sets the STUN and TURN servers new connections use
export function setIceServers(servers: RTCIceServer[]): void {
  ice_servers = servers;
}

// Enables or disables the microphone track on every connection
export function setMicrophoneEnabled(enabled: boolean): void {
  if (microphone === null) {
    return;
  }

  for (const track of microphone.getAudioTracks()) {
    track.enabled = enabled;
  }
}

// Mutes or unmutes every participant's audio, including connections made later
export function setAudioMuted(muted: boolean): void {
  audio_muted = muted;

  for (const [user_id, peer] of peers) {
    applyMuted(user_id, peer.audio);
  }
}

// Sets one participant's playback volume, from 0 to 1
export function setVolume(user_id: string, volume: number): void {
  volumes.set(user_id, volume);

  const peer = peers.get(user_id);

  if (peer !== undefined) {
    peer.audio.volume = volume;
  }
}

// Mutes or unmutes one participant's audio on this client
export function setUserMuted(user_id: string, muted: boolean): void {
  if (muted) {
    muted_users.add(user_id);
  } else {
    muted_users.delete(user_id);
  }

  const peer = peers.get(user_id);

  if (peer !== undefined) {
    applyMuted(user_id, peer.audio);
  }
}

// PEERS //

// Creates the connection to one participant, replacing any earlier one
export function create(user_id: string): Peer {
  remove(user_id);

  const connection = new RTCPeerConnection({ iceServers: ice_servers });
  const audio = new Audio();
  applyMuted(user_id, audio);

  // Keep participant's volume across reconnects
  const volume = volumes.get(user_id);

  if (volume !== undefined) {
    audio.volume = volume;
  }

  // Send microphone over connection
  if (microphone !== null) {
    for (const track of microphone.getAudioTracks()) {
      connection.addTrack(track, microphone);
    }
  }

  // Send each ICE candidate to participant
  connection.onicecandidate = (event: RTCPeerConnectionIceEvent) => {
    if (event.candidate !== null) {
      socket.sendSignal(user_id, { candidate: event.candidate.toJSON() });
    }
  };

  // Play participant's audio once it arrives
  connection.ontrack = async (event: RTCTrackEvent) => {
    audio.srcObject = event.streams[0];
    levels.watch(user_id, event.streams[0]);

    // Playback started before any user interaction is blocked by the browser
    try {
      await audio.play();
    } catch (error) {
      console.warn(error);
    }
  };

  // Remove participant once connection fails
  connection.onconnectionstatechange = () => {
    const current = peers.get(user_id);

    // Ignore a connection already replaced
    if (current === undefined || current.connection !== connection) {
      return;
    }

    if (connection.connectionState === "failed") {
      remove(user_id);
    }
  };

  const peer: Peer = { connection, audio };
  peers.set(user_id, peer);

  return peer;
}

// Closes the connection to one participant and stops their audio
export function remove(user_id: string): void {
  const peer = peers.get(user_id);

  if (peer === undefined) {
    return;
  }

  peer.connection.close();
  peer.audio.srcObject = null;
  levels.unwatch(user_id);
  peers.delete(user_id);
}

// Closes every connection and stops the microphone
export function clear(): void {
  for (const user_id of getUserIds()) {
    remove(user_id);
  }

  if (microphone !== null) {
    for (const track of microphone.getTracks()) {
      track.stop();
    }
  }

  microphone = null;
  ice_servers = [];
}

// HELPERS //

// Mutes a participant's audio when all sound or that participant is muted
function applyMuted(user_id: string, audio: HTMLAudioElement): void {
  audio.muted = audio_muted || muted_users.has(user_id);
}
