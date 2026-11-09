// The browser's RTC socket and its one WebRTC connection to the server
// https://www.w3.org/TR/webrtc/#rtcpeerconnection-interface

import type {
  RTCIceCandidateInit as IceCandidate,
  RTCSessionDescriptionInit as SessionDescription,
  RtcClientEvent,
  RtcServerEvent,
  RtcStream,
} from "@/bindings/rtc/events";
import { LocalEvent, RtcEventType } from "@/lib/events";
import * as events from "@/lib/events";
import * as server from "@/lib/connection/server.svelte";
import * as session from "@/lib/connection/session.svelte";

// The server's offer opens with the slots this browser sends in, in this order
const SLOT_MICROPHONE: number = 0;
const SLOT_SCREEN_AUDIO: number = 1;
const SLOT_SCREEN_VIDEO: number = 2;
const SLOTS_SENT: number = 3;

let socket: WebSocket | null = null;
let connection: RTCPeerConnection | null = null;
let microphone: MediaStream | null = null;
let screen: MediaStream | null = null;

// Candidates that arrived before the server's first offer was applied
let pending: IceCandidate[] = [];

// Streams from the others in this call
let streams: RtcStream[] = [];

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
  streams = [];

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

  const opened = new RTCPeerConnection({ iceServers: iceServers() });
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

// Closes the socket and the connection without reporting it, and stops sending
export function close(): void {
  if (socket !== null) {
    socket.onclose = null;
    socket.close();
  }

  connection?.close();
  microphone?.getTracks().forEach((track) => track.stop());
  screen?.getTracks().forEach((track) => track.stop());

  socket = null;
  connection = null;
  microphone = null;
  screen = null;
  pending = [];
  streams = [];
}

// The stream with this id
export function findStream(stream_id: string): RtcStream | undefined {
  return streams.find((entry) => entry.stream_id === stream_id);
}

// Enables or disables the microphone without renegotiating
export function setMicrophoneEnabled(enabled: boolean): void {
  microphone?.getAudioTracks().forEach((track) => track.enabled = enabled);
}

// Sends a screen share, or stops it when none is given, without renegotiating
export async function setScreen(stream: MediaStream | null): Promise<void> {
  screen?.getTracks().forEach((track) => track.stop());
  screen = stream;

  if (connection !== null && connection.remoteDescription !== null) {
    await fillSlots(connection);
  }
}

// Applies the screen share's resolution, framerate and bitrate cap to the
// active share, live, without renegotiating
export async function applyScreenSettings(
  width: number,
  height: number,
  frame_rate: number,
  bitrate_kbps: number,
): Promise<void> {
  const track = screen?.getVideoTracks()[0];

  if (track === undefined) {
    return;
  }

  // Tell the encoder the content moves, so it keeps the framerate over detail
  // https://www.w3.org/TR/mst-content-hint/#video-content-hints
  track.contentHint = "motion";

  // The encoder downscales the capture to fit these sizes
  // https://www.w3.org/TR/mediacapture-main/#dom-mediastreamtrack-applyconstraints
  await track.applyConstraints({
    width: { ideal: width },
    height: { ideal: height },
    frameRate: { ideal: frame_rate },
  });

  // Cap the sender's bitrate, when a cap is set
  // https://www.w3.org/TR/webrtc/#dom-rtcrtpsender-setparameters
  if (connection === null) {
    return;
  }

  const sender = connection.getSenders().find((entry) => entry.track === track);

  if (sender === undefined) {
    return;
  }

  const parameters = sender.getParameters();

  let encodings = parameters.encodings;
  if (encodings.length === 0) {
    encodings = [{}];
  }

  if (bitrate_kbps > 0) {
    encodings[0].maxBitrate = bitrate_kbps * 1000;
  } else {
    encodings[0].maxBitrate = undefined;
  }

  await sender.setParameters({
    ...parameters,
    encodings,
    degradationPreference: "maintain-framerate",
  });
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
      streams = event.streams;
      await receiveOffer(event.description);
      break;

    case RtcEventType.candidate:
      await receiveCandidate(event.candidate);
      break;

    case RtcEventType.screen:
      events.dispatch(LocalEvent.screen_shared, event);
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
  await fillSlots(connection);

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

// Every slot this browser sends in sends, so a track can fill one later without a new offer
async function fillSlots(opened: RTCPeerConnection): Promise<void> {
  const slots = opened.getTransceivers().slice(0, SLOTS_SENT);
  slots.forEach((slot) => slot.direction = "sendonly");

  await fillSlot(slots[SLOT_MICROPHONE], microphone?.getAudioTracks()[0] ?? null);
  await fillSlot(slots[SLOT_SCREEN_AUDIO], screen?.getAudioTracks()[0] ?? null);
  await fillSlot(slots[SLOT_SCREEN_VIDEO], screen?.getVideoTracks()[0] ?? null);
}

// https://www.w3.org/TR/webrtc/#dom-rtcrtpsender-replacetrack
async function fillSlot(
  slot: RTCRtpTransceiver | undefined,
  track: MediaStreamTrack | null,
): Promise<void> {
  if (slot !== undefined && slot.sender.track !== track) {
    await slot.sender.replaceTrack(track);
  }
}

// The server's STUN server, when one is set
// https://www.w3.org/TR/webrtc/#dom-rtcconfiguration-iceservers
function iceServers(): RTCIceServer[] {
  const stun_url = server.getInfo()?.stun_url ?? null;

  if (stun_url === null) {
    return [];
  }

  return [{ urls: stun_url }];
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
