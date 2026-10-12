// Shared behaviour for the hand-drawn SVG characters: cursor-tracking
// pupils, idle blinking, and occasional idle gestures.
//
// Each character used to carry its own copy of this (~100 near-identical
// lines). Extracting it keeps a new character to just its artwork, and
// means a fix here lands for every character at once.

import { onDestroy, onMount } from "svelte";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { SceneState } from "./director";

interface CursorPositionPayload {
  x: number;
  y: number;
}

export interface BehaviourOptions {
  /** Live scene state — read as a getter so the caller's `$props` value
   * stays reactive rather than being captured once at setup. */
  sceneState: () => SceneState;
  /** True when the parent mirrored the character, so eye tracking can
   * un-mirror the cursor direction before computing pupil offsets. */
  mirrored: () => boolean;
  /** The character's root element, for measuring where it is on screen. */
  rootEl: () => HTMLElement | undefined;
  /** Idle gestures this character supports. Empty = no idle gestures. */
  idleGestures?: readonly string[];
  /** How far pupils travel from centre, in SVG units. */
  maxPupilOffset?: number;
  /** Vertical position of the eyes within the character's box (0-1),
   * used as the origin for cursor-direction maths. */
  eyeHeightRatio?: number;
}

export interface CharacterBehaviour {
  readonly pupilOffset: { x: number; y: number };
  readonly blinking: boolean;
  readonly idleGesture: string;
  readonly eyesClosed: boolean;
  readonly isWalking: boolean;
}

/**
 * Wires up the shared character behaviour. Call once from a character
 * component's `<script>`; the returned object's fields are reactive.
 */
export function useCharacterBehaviour(options: BehaviourOptions): CharacterBehaviour {
  const {
    sceneState,
    mirrored,
    rootEl,
    idleGestures = [],
    maxPupilOffset = 3,
    eyeHeightRatio = 0.3,
  } = options;

  let pupilOffset = $state({ x: 0, y: 0 });
  let blinking = $state(false);
  let idleGesture = $state("none");

  let blinkTimer: ReturnType<typeof setTimeout> | undefined;
  let idleTimer: ReturnType<typeof setTimeout> | undefined;
  let gestureResetTimer: ReturnType<typeof setTimeout> | undefined;
  let unlistenCursor: (() => void) | undefined;

  function onCursorPosition(pos: CursorPositionPayload) {
    const el = rootEl();
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height * eyeHeightRatio;
    const dx = (pos.x - cx) * (mirrored() ? -1 : 1);
    const dy = pos.y - cy;
    const dist = Math.hypot(dx, dy) || 1;
    pupilOffset = {
      x: (dx / dist) * Math.min(maxPupilOffset, dist / 40),
      y: (dy / dist) * Math.min(maxPupilOffset, dist / 40),
    };
  }

  function scheduleBlink() {
    const delay = 2200 + Math.random() * 3800;
    blinkTimer = setTimeout(async () => {
      blinking = true;
      await new Promise((r) => setTimeout(r, 120));
      blinking = false;
      scheduleBlink();
    }, delay);
  }

  // Procedural idle variety: every few seconds, while idle, play one
  // small random gesture so standing still never reads as a static
  // looping GIF. Only one plays at a time and it fully resets before the
  // next is picked.
  function scheduleIdleGesture() {
    const delay = 3200 + Math.random() * 4600;
    idleTimer = setTimeout(() => {
      if (sceneState() === "idle" && idleGestures.length > 0) {
        idleGesture = idleGestures[Math.floor(Math.random() * idleGestures.length)];
        gestureResetTimer = setTimeout(() => {
          idleGesture = "none";
        }, 1400);
      }
      scheduleIdleGesture();
    }, delay);
  }

  onMount(() => {
    getCurrentWebviewWindow()
      .listen<CursorPositionPayload>("overlay://cursor", (event) =>
        onCursorPosition(event.payload),
      )
      .then((unlisten) => {
        unlistenCursor = unlisten;
      });
    scheduleBlink();
    if (idleGestures.length > 0) scheduleIdleGesture();
  });

  onDestroy(() => {
    unlistenCursor?.();
    if (blinkTimer) clearTimeout(blinkTimer);
    if (idleTimer) clearTimeout(idleTimer);
    if (gestureResetTimer) clearTimeout(gestureResetTimer);
  });

  return {
    get pupilOffset() {
      return pupilOffset;
    },
    get blinking() {
      return blinking;
    },
    get idleGesture() {
      return idleGesture;
    },
    get eyesClosed() {
      return blinking || sceneState() === "sad";
    },
    get isWalking() {
      const s = sceneState();
      return s === "walk_in" || s === "walk_out";
    },
  };
}
