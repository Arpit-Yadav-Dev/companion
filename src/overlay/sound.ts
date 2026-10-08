// Sound effects for the overlay: a soft "pop" on reminder entry and a
// "gulp/ding" on YES.
//
// PLACEHOLDER NOTICE: these are synthesized with the Web Audio API rather
// than real recorded sound-design assets (.ogg/.m4a), because no audio
// files were available when this was built. The brief calls for bundled
// recorded sounds, and the user has confirmed sound is a required part of
// the final polish, not optional — swapping these synthesized tones for
// real short audio files is a known, tracked follow-up (see memory:
// sound-assets-followup), not a finished design decision.

let audioContext: AudioContext | null = null;

function getContext(): AudioContext {
  if (!audioContext) {
    audioContext = new AudioContext();
  }
  return audioContext;
}

function playTone(
  frequencies: number[],
  durationMs: number,
  volume: number,
  type: OscillatorType = "sine",
) {
  const ctx = getContext();
  if (ctx.state === "suspended") {
    void ctx.resume();
  }

  const now = ctx.currentTime;
  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0, now);
  gain.gain.linearRampToValueAtTime(volume, now + 0.015);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + durationMs / 1000);
  gain.connect(ctx.destination);

  const stepDuration = durationMs / 1000 / frequencies.length;
  frequencies.forEach((freq, i) => {
    const osc = ctx.createOscillator();
    osc.type = type;
    osc.frequency.value = freq;
    osc.connect(gain);
    const start = now + i * stepDuration;
    osc.start(start);
    osc.stop(start + stepDuration + 0.02);
  });
}

/** Soft, short pop — played when the character walks in / the bubble appears. */
export function playEntryPop(volume: number) {
  playTone([520], 110, volume * 0.5, "sine");
}

/** A cheerful rising "gulp/ding" — played on YES. */
export function playYesDing(volume: number) {
  playTone([660, 880], 220, volume * 0.6, "sine");
}
