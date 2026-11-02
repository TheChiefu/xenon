// The browser's RTC socket and its one WebRTC connection to the server
// https://www.w3.org/TR/webrtc/#rtcpeerconnection-interface

import type {
  RTCIceCandidateInit as IceCandidate,
  RTCSessionDescriptionInit as SessionDescription,
  RtcClientEvent,
  RtcServerEvent,
} from "@/bindings/rtc/events";
import { RtcEventType } from "@/lib/events";
import * as session from "@/lib/connection/session.svelte";

// The server's offer opens with the slots this browser sends in, in this order
const SLOT_MICROPHONE: number = 0;
const SLOTS_SENT: number = 1;

let socket: WebSocket | null = null;
let connection: RTCPeerConnection | null = null;
let microphone: MediaStream | null = null;

// Candidates that arrived before the server's first offer was applied
let pending: IceCandidate[] = [];

// CONNECTION //

// Opens the RTC socket for a room, which joins it, and the connection the server offers
export function open(
  roomId: string,
  stream: MediaStream,
  onTrack: (event: RTCTrackEvent) => void,
  onClosed: (code: number) => void,
): void {
  const url = session.getUrl();
  const token = session.getToken();

  if (url === null || token === null) {
    return;
  }

  microphone = stream;
  pending = [];

  const address = new URL(`${url}/rooms/${roomId}/rtc`);
  address.protocol = address.protocol === "https:" ? "wss:" : "ws:";

  const opened_socket = new WebSocket(address, ["Bearer", token]);
  socket = opened_socket;

  opened_socket.onclose = (closed: CloseEvent) => onClosed(closed.code);
  opened_socket.onmessage = (message: MessageEvent) => {
    if (typeof message.data === "string") {
      receive(JSON.parse(message.data)).catch(console.warn);
    }
  };

  const opened = new RTCPeerConnection();
  connection = opened;

  // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onicecandidate
  opened.onicecandidate = (event: RTCPeerConnectionIceEvent) => {
    if (event.candidate === null) {
      return;
    }

    send({
      type: RtcEventType.candidate,
      candidate: {
        candidate: event.candidate.candidate,
        sdpMid: event.candidate.sdpMid,
        sdpMLineIndex: event.candidate.sdpMLineIndex,
        usernameFragment: event.candidate.usernameFragment,
      },
    });
  };

  // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-ontrack
  opened.ontrack = onTrack;

  // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onconnectionstatechange
  opened.onconnectionstatechange = () => {
    if (opened.connectionState === "failed") {
      onClosed(0);
    }
  };
}

// Closes the socket and the connection without reporting it, and stops the microphone
export function close(): void {
  if (socket !== null) {
    socket.onclose = null;
    socket.close();
  }

  connection?.close();
  microphone?.getTracks().forEach((track) => track.stop());

  socket = null;
  connection = null;
  microphone = null;
  pending = [];
}

// Enables or disables the microphone without renegotiating
export function setMicrophoneEnabled(enabled: boolean): void {
  microphone?.getAudioTracks().forEach((track) => track.enabled = enabled);
}

export function send(event: RtcClientEvent): void {
  if (socket?.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify(event));
  }
}

// HELPERS //

async function receive(event: RtcServerEvent): Promise<void> {
  switch (event.type) {
    case RtcEventType.offer:
      await receiveOffer(event.description);
      break;

    case RtcEventType.candidate:
      await receiveCandidate(event.candidate);
      break;
  }
}

// Applies the server's offer and answers it
// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-setremotedescription
async function receiveOffer(description: SessionDescription): Promise<void> {
  if (connection === null) {
    return;
  }

  await connection.setRemoteDescription(description);

  // Every slot this browser sends in sends, so a track can fill one later without a new offer
  const slots = connection.getTransceivers().slice(0, SLOTS_SENT);
  slots.forEach((slot) => slot.direction = "sendonly");

  const track = microphone?.getAudioTracks()[0] ?? null;
  const slot = slots[SLOT_MICROPHONE];
  if (slot !== undefined && slot.sender.track !== track) {
    await slot.sender.replaceTrack(track);
  }

  for (const candidate of pending) {
    await connection.addIceCandidate(candidate);
  }
  pending = [];

  const answer = await connection.createAnswer();
  await connection.setLocalDescription(answer);

  send({
    type: RtcEventType.answer,
    description: { type: "answer", sdp: answer.sdp ?? "" },
  });
}

// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-addicecandidate
async function receiveCandidate(candidate: IceCandidate): Promise<void> {
  if (connection === null) {
    return;
  }

  if (connection.remoteDescription === null) {
    pending.push(candidate);
    return;
  }

  await connection.addIceCandidate(candidate);
}
