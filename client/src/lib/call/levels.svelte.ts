import { SvelteSet } from "svelte/reactivity";

// Audio stream feeding an analyser, which reports loudness without playing it
interface Meter {
  source: MediaStreamAudioSourceNode;
  analyser: AnalyserNode;
}

// Loudness, from 0 to 1, above which a participant counts as speaking
const SPEAKING_LEVEL: number = 0.02;

// Time a participant stays speaking after their last loud moment
const HOLD_MS: number = 300;

const MEASURE_INTERVAL_MS: number = 100;

// STATE //

let context: AudioContext | null = null;
let timer: number | null = null;
const meters: Map<string, Meter> = new Map();

// Time each participant was last loud, keyed by user id
const last_loud: Map<string, number> = new Map();

const speaking: SvelteSet<string> = new SvelteSet();

// GETTERS //

export function isSpeaking(user_id: string): boolean {
  return speaking.has(user_id);
}

// METERS //

// Starts measuring a participant's audio stream, replacing any earlier one
export function watch(user_id: string, stream: MediaStream): void {
  unwatch(user_id);

  if (context === null) {
    context = new AudioContext();
  }

  const source = context.createMediaStreamSource(stream);
  const analyser = context.createAnalyser();
  source.connect(analyser);

  meters.set(user_id, { source, analyser });

  // Start measuring with the first meter
  if (timer === null) {
    timer = window.setInterval(measure, MEASURE_INTERVAL_MS);
  }
}

// Stops measuring a participant's audio stream
export function unwatch(user_id: string): void {
  const meter = meters.get(user_id);

  if (meter === undefined) {
    return;
  }

  meter.source.disconnect();
  meters.delete(user_id);
  last_loud.delete(user_id);
  speaking.delete(user_id);
}

// Stops measuring every audio stream
export function clear(): void {
  for (const user_id of [...meters.keys()]) {
    unwatch(user_id);
  }

  if (timer !== null) {
    window.clearInterval(timer);
    timer = null;
  }

  if (context !== null) {
    context.close();
    context = null;
  }
}

// HELPERS //

// Marks each participant speaking or silent from their latest loudness
function measure(): void {
  const now = performance.now();

  for (const [user_id, meter] of meters) {
    if (loudness(meter.analyser) > SPEAKING_LEVEL) {
      last_loud.set(user_id, now);
    }

    const heard = last_loud.get(user_id);

    if (heard !== undefined && now - heard < HOLD_MS) {
      speaking.add(user_id);
    } else {
      speaking.delete(user_id);
    }
  }
}

// Root mean square of the analyser's latest samples, from 0 to 1
function loudness(analyser: AnalyserNode): number {
  const samples = new Float32Array(analyser.fftSize);
  analyser.getFloatTimeDomainData(samples);

  let sum = 0;

  for (const sample of samples) {
    sum += sample * sample;
  }

  return Math.sqrt(sum / samples.length);
}
