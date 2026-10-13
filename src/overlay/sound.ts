// Sound effects for the overlay: a soft "pop" on entry, a celebratory
// clap on YES, a gentle neutral sound on "later", and a bigger fanfare
// when the daily goal is reached.
//
// Real recorded assets (static/sounds/*.wav), not synthesized tones —
// see static/sounds/LICENSE.txt for each file's source and license (all
// CC0/free for commercial use, no attribution required).

const ENTRY_POP_SRC = "/sounds/entry-pop.wav";
const YES_CLAP_SRC = "/sounds/yes-clap.wav";
const LATER_SHRUG_SRC = "/sounds/later-shrug.wav";
const GOAL_REACHED_SRC = "/sounds/goal-reached.wav";

function play(src: string, volume: number) {
  // A fresh Audio element per play, rather than cloneNode on a shared
  // preloaded element — cloneNode's clone doesn't reliably inherit the
  // original's already-buffered data in WKWebView, so it can end up
  // re-fetching/re-decoding on every play and silently never actually
  // starting in time (or at all). These files are tiny, so a fresh
  // element each time costs nothing and this also naturally lets the
  // same sound overlap itself if two reminders fire close together.
  const instance = new Audio(src);
  instance.volume = Math.min(Math.max(volume, 0), 1);
  void instance.play().catch(() => {
    // Autoplay/permission errors are not actionable here — the overlay
    // has no visible UI to surface them in, and a missed sound effect
    // isn't worth interrupting the reminder flow over.
  });
}

// The files are mastered to within a few dB of each other, so these
// multipliers only express intent — the entry and "later" cues are
// deliberately softer than the rewards, since they interrupt rather than
// celebrate. They are not correcting for mismatched source levels, which
// is what the old wider spread (0.6-0.85) was doing.

/** Soft, short pop — played when the character walks in / the bubble appears. */
export function playEntryPop(volume: number) {
  play(ENTRY_POP_SRC, volume * 0.75);
}

/** A short, fun round of applause — played on YES. */
export function playYesDing(volume: number) {
  play(YES_CLAP_SRC, volume);
}

/** A gentle, neutral acknowledgment — played on "Remind me later". */
export function playLaterShrug(volume: number) {
  play(LATER_SHRUG_SRC, volume * 0.7);
}

/** A bigger celebratory fanfare — played when the daily goal is reached. */
export function playGoalReached(volume: number) {
  play(GOAL_REACHED_SRC, volume);
}
