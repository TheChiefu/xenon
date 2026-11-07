import { Media } from "@/bindings/rtc/events";
import type { RtcCloseCode } from "@/bindings/rtc/events";
import { LocalEvent, RtcEventType, ServerEventType } from "@/lib/events";
import * as events from "@/lib/events";
import * as connection from "@/lib/rtc/connection.svelte";
import * as levels from "@/lib/rtc/levels.svelte";
import * as microphones from "@/lib/rtc/microphones.svelte";
import * as screens from "@/lib/rtc/screens.svelte";
import * as sounds from "@/lib/rtc/sounds";
import * as screen_share from "@/lib/client/screen_share.svelte";
import * as profile from "@/lib/users/profile.svelte";
import { LABELS_RTC_CLOSE, MSG_MICROPHONE_DENIED } from "@/lib/labels";

// STATE //

let room_id: string | null = $state(null);
let failure: string | null = $state(null);
let mic_enabled: boolean = $state(true);
let screen_enabled: boolean = $state(false);

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

export function getScreenEnabled(): boolean {
  return screen_enabled;
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
  if (room_id !== null) {
    sounds.playLeave();
  }

  connection.close();
  microphones.clear();
  screens.clear();
  levels.clear();

  room_id = null;
  screen_enabled = false;
}

// SETTINGS //

export function setMicEnabled(enabled: boolean): void {
  mic_enabled = enabled;
  connection.setMicrophoneEnabled(enabled);
}

// Shares a screen the user picks, or stops sharing
// https://www.w3.org/TR/screen-capture/#dom-mediadevices-getdisplaymedia
export async function setScreenEnabled(enabled: boolean): Promise<void> {
  if (!enabled) {
    screen_enabled = false;
    await connection.setScreen(null);
    connection.send({ type: RtcEventType.screen, sharing: false });
    return;
  }

  let screen: MediaStream;

  try {
    screen = await navigator.mediaDevices.getDisplayMedia({
      video: {
        width: { ideal: screen_share.getWidth() },
        height: { ideal: screen_share.getHeight() },
        frameRate: { ideal: screen_share.getFrameRate() },
      },
      audio: true,
    });
  } catch {
    return;
  }

  // Sharing stopped from the browser's own controls
  screen.getVideoTracks()[0]?.addEventListener("ended", () => {
    setScreenEnabled(false).catch(console.warn);
  });

  screen_enabled = true;
  await connection.setScreen(screen);
  await connection.applyScreenSettings(
    screen_share.getWidth(),
    screen_share.getHeight(),
    screen_share.getFrameRate(),
    screen_share.getBitrate(),
  );
  connection.send({ type: RtcEventType.screen, sharing: true });
}

// HELPERS //

// Hands a participant's incoming stream to the microphones or the screens
function play(event: RTCTrackEvent): void {
  const stream = event.streams[0];

  if (stream === undefined) {
    return;
  }

  const found = connection.findStream(stream.id);

  if (found === undefined) {
    return;
  }

  if (found.media === Media.microphone) {
    microphones.play(found.user_id, stream);
  } else {
    screens.add(found.user_id, stream);
  }
}

// Ends the session after the RTC socket closes or the connection fails, showing why when known
function closed(code: number): void {
  failure = LABELS_RTC_CLOSE[code as RtcCloseCode] ?? null;
  leave();
}

// LISTENERS //

// Applies changed screen share settings to the active share, live
events.subscribe(LocalEvent.screen_share_settings_changed, () => {
  if (!screen_enabled) {
    return;
  }

  connection.applyScreenSettings(
    screen_share.getWidth(),
    screen_share.getHeight(),
    screen_share.getFrameRate(),
    screen_share.getBitrate(),
  ).catch(console.warn);
});

// A participant joined this call, or this user joined one
events.subscribe(ServerEventType.rtc_joined, (event) => {
  if (event.room_id === room_id) {
    sounds.playJoin();
  }
});

// Stop playing a participant who left this call
events.subscribe(ServerEventType.rtc_left, (event) => {
  if (event.room_id === room_id) {
    microphones.stop(event.user_id);
    screens.stop(event.user_id);
    sounds.playLeave();
  }
});
