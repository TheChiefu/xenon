// Audio input devices, and which one the call's microphone uses

// MICROPHONE DEVICE //
const keyMicDevice = "mic_device_id";

let currentMicDevice: string | null = $state(localStorage.getItem(keyMicDevice));

export function getMicDevice(): string | null {
  return currentMicDevice;
}

export function setMicDevice(device_id: string | null): void {
  currentMicDevice = device_id;

  if (device_id === null) {
    localStorage.removeItem(keyMicDevice);
  } else {
    localStorage.setItem(keyMicDevice, device_id);
  }
}

// CAPTURE //

// The call's microphone, falling back to the default device when the saved one is gone
export async function captureMic(): Promise<MediaStream> {
  const device = getMicDevice();

  if (device !== null) {
    try {
      return await navigator.mediaDevices.getUserMedia({
        audio: { deviceId: { exact: device } },
      });
    } catch {
      // Saved device unavailable, fall through to the default
    }
  }

  return navigator.mediaDevices.getUserMedia({ audio: true });
}

// LISTING //

// The audio input devices the browser exposes, for the device dropdowns
// https://www.w3.org/TR/mediacapture-main/#dom-mediadevices-enumeratedevices
export async function listAudioInputs(): Promise<MediaDeviceInfo[]> {
  const devices = await navigator.mediaDevices.enumerateDevices();
  return devices.filter((device) => device.kind === "audioinput");
}
