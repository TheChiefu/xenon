// The browser's one WebRTC connection to the server
// https://www.w3.org/TR/webrtc/#rtcpeerconnection-interface

import type { RTCIceCandidateInit as IceCandidate, RTCSessionDescriptionInit as SessionDescription } from "@/bindings/rtc/events";
import { RtcEventType } from "@/lib/events";
import * as socket from "@/lib/rtc/socket.svelte";

// The server's offer opens with the slots this browser sends in, in this order
const SLOT_MICROPHONE: number = 0;
const SLOTS_SENT: number = 4;

let connection: RTCPeerConnection | null = null;
let microphone: MediaStream | null = null;

// Candidates that arrived before the server's first offer was applied
let pending: IceCandidate[] = [];

// CONNECTION //

// Opens the connection, sending the microphone once the server offers its slot
export function open(
  stream: MediaStream,
  onTrack: (event: RTCTrackEvent) => void,
  onFailed: () => void,
): void {
  microphone = stream;
  pending = [];

  const opened = new RTCPeerConnection();
  connection = opened;

  // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onicecandidate
  opened.onicecandidate = (event: RTCPeerConnectionIceEvent) => {
    if (event.candidate === null) {
      return;
    }

    const candidate: IceCandidate = {
      candidate: event.candidate.candidate,
      sdpMid: event.candidate.sdpMid,
      sdpMLineIndex: event.candidate.sdpMLineIndex,
      usernameFragment: event.candidate.usernameFragment,
    };

    socket.send({ type: RtcEventType.candidate, candidate });
  };

  // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-ontrack
  opened.ontrack = onTrack;

  opened.oniceconnectionstatechange = () => {
    console.debug(new Date().toISOString(), "rtc ice state:", opened.iceConnectionState);
  };

  // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onconnectionstatechange
  opened.onconnectionstatechange = () => {
    console.debug(new Date().toISOString(), "rtc connection state:", opened.connectionState);

    if (opened.connectionState === "failed") {
      onFailed();
    }
  };
}

export function close(): void {
  connection?.close();
  connection = null;
  pending = [];

  if (microphone !== null) {
    for (const track of microphone.getTracks()) {
      track.stop();
    }
  }

  microphone = null;
}

// Enables or disables the microphone without renegotiating
export function setMicrophoneEnabled(enabled: boolean): void {
  if (microphone === null) {
    return;
  }

  for (const track of microphone.getAudioTracks()) {
    track.enabled = enabled;
  }
}

// SIGNALS //

// Applies the server's offer and answers it
// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-setremotedescription
export async function receiveOffer(description: SessionDescription): Promise<void> {
  if (connection === null) {
    return;
  }

  await connection.setRemoteDescription(description);
  await fillSlots(connection);

  for (const candidate of pending) {
    await connection.addIceCandidate(candidate);
  }
  pending = [];

  const answer = await connection.createAnswer();
  await connection.setLocalDescription(answer);

  socket.send({
    type: RtcEventType.answer,
    description: { type: "answer", sdp: answer.sdp ?? "" },
  });
}

// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-addicecandidate
export async function receiveCandidate(candidate: IceCandidate): Promise<void> {
  if (connection === null) {
    return;
  }

  if (connection.remoteDescription === null) {
    pending.push(candidate);
    return;
  }

  await connection.addIceCandidate(candidate);
}

// HELPERS //

// Marks every slot this browser sends in as sending, so a track can fill one later
// without a new offer, and fills the microphone slot
async function fillSlots(opened: RTCPeerConnection): Promise<void> {
  const slots = opened.getTransceivers().slice(0, SLOTS_SENT);

  for (const slot of slots) {
    slot.direction = "sendonly";
  }

  const track = microphone?.getAudioTracks()[0] ?? null;
  const slot = slots[SLOT_MICROPHONE];

  if (slot !== undefined && slot.sender.track !== track) {
    await slot.sender.replaceTrack(track);
  }
}
