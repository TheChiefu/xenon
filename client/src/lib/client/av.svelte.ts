let micEnabled = $state(true);
let soundEnabled = $state(true);

export function getMicEnabled(): boolean {
  return micEnabled;
}

export function getSoundEnabled(): boolean {
  return soundEnabled;
}

export async function setMicEnabled(value: boolean): Promise<void> {
  micEnabled = value;
}

export function setSoundEnabled(value: boolean): void {
  soundEnabled = value;
}
