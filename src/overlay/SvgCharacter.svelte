<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import type { SceneState } from "./director";

  interface CursorPositionPayload {
    x: number;
    y: number;
  }

  interface Props {
    sceneState: SceneState;
    /** 1 = default size; the Character tab's size slider (60%-140%) multiplies this. */
    scale?: number;
    theme?: "blue" | "peach" | "mint" | "night";
    /** True when the parent has mirrored this character with scaleX(-1)
     * (e.g. entering from the right), so eye-tracking math can un-mirror
     * the cursor direction before computing pupil offsets. */
    mirrored?: boolean;
  }

  let { sceneState, scale = 1, theme = "blue", mirrored = false }: Props = $props();

  let rootEl: HTMLDivElement | undefined = $state();

  // --- Eye tracking: pupils lean a few px toward the cursor's position
  // relative to the character's own screen position.
  //
  // This listens for the `overlay://cursor` broadcast from Rust's
  // hit-region polling loop rather than a DOM `mousemove` listener,
  // because the overlay window is click-through (set_ignore_cursor_events)
  // everywhere except directly over the character/bubble — so it only
  // ever receives native mousemove events while the cursor happens to
  // already be over the character, which made the eyes track only after
  // the first hover/click. The Rust-side global cursor poll has no such
  // gap. ---
  let pupilOffset = $state({ x: 0, y: 0 });

  function onCursorPosition(pos: CursorPositionPayload) {
    if (!rootEl) return;
    const rect = rootEl.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height * 0.35; // roughly eye height
    // getBoundingClientRect() already reflects the mirrored (scaleX(-1))
    // layout box, so dx is correct in screen space — but the SVG's own
    // coordinate system is flipped relative to the screen when mirrored,
    // so the offset applied to cx/cy inside that SVG needs negating.
    const dx = (pos.x - cx) * (mirrored ? -1 : 1);
    const dy = pos.y - cy;
    const dist = Math.hypot(dx, dy) || 1;
    const maxOffset = 3.5;
    pupilOffset = {
      x: (dx / dist) * Math.min(maxOffset, dist / 40),
      y: (dy / dist) * Math.min(maxOffset, dist / 40),
    };
  }

  // --- Blinking: random interval 2-6s, occasional double-blink. ---
  let blinking = $state(false);
  let blinkTimer: ReturnType<typeof setTimeout> | undefined;

  function scheduleBlink() {
    const delay = 2000 + Math.random() * 4000;
    blinkTimer = setTimeout(async () => {
      blinking = true;
      await new Promise((r) => setTimeout(r, 120));
      blinking = false;

      if (Math.random() < 0.25) {
        await new Promise((r) => setTimeout(r, 140));
        blinking = true;
        await new Promise((r) => setTimeout(r, 120));
        blinking = false;
      }

      scheduleBlink();
    }, delay);
  }

  let unlistenCursor: (() => void) | undefined;

  onMount(() => {
    const webview = getCurrentWebviewWindow();
    webview
      .listen<CursorPositionPayload>("overlay://cursor", (event) =>
        onCursorPosition(event.payload),
      )
      .then((unlisten) => {
        unlistenCursor = unlisten;
      });

    scheduleBlink();
  });

  onDestroy(() => {
    unlistenCursor?.();
    if (blinkTimer) clearTimeout(blinkTimer);
  });

  let eyesClosed = $derived(blinking || sceneState === "sad");

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 86 150 Q 100 166 114 150",
    shrug: "M 90 152 Q 100 150 110 152",
    sad: "M 88 154 Q 100 146 112 154",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 88 148 Q 100 158 112 148");
</script>

<div
  class="sip theme-{theme} state-{sceneState}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  {#if sceneState === "walk_in" || sceneState === "walk_out"}
    <div class="dust-puffs">
      <span class="puff p1"></span>
      <span class="puff p2"></span>
      <span class="puff p3"></span>
    </div>
  {/if}

  <svg
    class="body-svg"
    viewBox="0 0 200 220"
    width="160"
    height="176"
    aria-hidden="true"
  >
    <!-- Droplet body -->
    <path
      class="droplet"
      d="M100 10
         C 100 10, 40 95, 40 145
         C 40 183, 67 208, 100 208
         C 133 208, 160 183, 160 145
         C 160 95, 100 10, 100 10
         Z"
    />

    <!-- Glossy highlight -->
    <ellipse class="highlight" cx="72" cy="90" rx="16" ry="24" />

    <!-- Stubby arms -->
    <ellipse class="arm arm-left" cx="38" cy="150" rx="16" ry="11" />
    <ellipse class="arm arm-right" cx="162" cy="150" rx="16" ry="11" />

    <!-- Stubby feet -->
    <ellipse class="foot foot-left" cx="78" cy="206" rx="16" ry="9" />
    <ellipse class="foot foot-right" cx="122" cy="206" rx="16" ry="9" />

    <!-- Blush -->
    <ellipse class="blush blush-left" cx="62" cy="150" rx="12" ry="7" />
    <ellipse class="blush blush-right" cx="138" cy="150" rx="12" ry="7" />

    <!-- Eyes -->
    <g class="eye eye-left" class:closed={eyesClosed}>
      <ellipse class="eye-white" cx="78" cy="118" rx="14" ry="17" />
      <circle
        class="pupil"
        cx={78 + pupilOffset.x}
        cy={118 + pupilOffset.y}
        r="6.5"
      />
      <path class="eyelid" d="M 64 118 Q 78 118 92 118" />
    </g>
    <g class="eye eye-right" class:closed={eyesClosed}>
      <ellipse class="eye-white" cx="122" cy="118" rx="14" ry="17" />
      <circle
        class="pupil"
        cx={122 + pupilOffset.x}
        cy={118 + pupilOffset.y}
        r="6.5"
      />
      <path class="eyelid" d="M 108 118 Q 122 118 136 118" />
    </g>

    <!-- Eyebrows (ask = raised, shrug/sad = flat/droopy) -->
    <path class="brow brow-left" d="M 65 98 Q 78 90 91 98" />
    <path class="brow brow-right" d="M 109 98 Q 122 90 135 98" />

    <!-- Mouth -->
    <path class="mouth" d={mouthPath} />

    <!-- One tiny tear for the sad state -->
    <ellipse class="tear" cx="92" cy="134" rx="3" ry="5" />

    <!-- Water-bottle prop for the "ask" state -->
    <g class="prop-bottle">
      <rect x="150" y="100" width="18" height="34" rx="4" />
      <rect x="156" y="92" width="6" height="10" rx="2" />
    </g>

    <!-- Sparkle burst for the happy state -->
    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s3" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .sip {
    --scale: 1;
    --droplet-fill: #6fc3ff;
    --droplet-fill-dark: #3f9fe0;
    --droplet-stroke: #2680c2;
    --blush: #ff9ec7;
    --mouth: #2b5e7a;

    position: relative;
    width: calc(160px * var(--scale));
    height: calc(176px * var(--scale));
    transform-origin: bottom center;
  }

  .sip.theme-peach {
    --droplet-fill: #ffb199;
    --droplet-fill-dark: #ff8d6b;
    --droplet-stroke: #e0623f;
  }
  .sip.theme-mint {
    --droplet-fill: #8de9c6;
    --droplet-fill-dark: #4fd1a5;
    --droplet-stroke: #2aa37e;
  }
  .sip.theme-night {
    --droplet-fill: #6b7bdb;
    --droplet-fill-dark: #4556b8;
    --droplet-stroke: #2e3a8f;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    animation: breathe 2.5s ease-in-out infinite;
    transform-origin: 50% 100%;
  }

  @keyframes breathe {
    0%,
    100% {
      transform: scaleX(1) scaleY(1);
    }
    50% {
      transform: scaleX(1.03) scaleY(0.97);
    }
  }

  .droplet {
    fill: var(--droplet-fill);
    stroke: var(--droplet-stroke);
    stroke-width: 3;
  }

  .highlight {
    fill: white;
    opacity: 0.55;
  }

  .arm,
  .foot {
    fill: var(--droplet-fill-dark);
    stroke: var(--droplet-stroke);
    stroke-width: 2;
  }

  .blush {
    fill: var(--blush);
    opacity: 0.6;
  }

  .eye-white {
    fill: white;
    stroke: var(--droplet-stroke);
    stroke-width: 1.5;
  }

  .pupil {
    fill: #233;
    transition:
      cx 80ms linear,
      cy 80ms linear;
  }

  .eyelid {
    fill: none;
    stroke: var(--droplet-stroke);
    stroke-width: 3;
    stroke-linecap: round;
    opacity: 0;
  }

  .eye.closed .eye-white,
  .eye.closed .pupil {
    opacity: 0;
  }
  .eye.closed .eyelid {
    opacity: 1;
  }

  .brow {
    fill: none;
    stroke: var(--droplet-stroke);
    stroke-width: 3.5;
    stroke-linecap: round;
    transform-origin: center;
    transition: transform 220ms ease;
  }

  .mouth {
    fill: none;
    stroke: var(--mouth);
    stroke-width: 4;
    stroke-linecap: round;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-bottle {
    opacity: 0;
    transform-origin: 159px 117px;
  }
  .prop-bottle rect:first-child {
    fill: #bfe6ff;
    stroke: var(--droplet-stroke);
    stroke-width: 2;
  }
  .prop-bottle rect:last-child {
    fill: #2680c2;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(40px, 60px);
  }
  .s2 {
    transform: translate(165px, 80px);
  }
  .s3 {
    transform: translate(100px, 30px);
  }

  /* ---------- State-driven expressions ---------- */

  .state-ask .brow {
    transform: translateY(-4px);
  }
  .state-ask .prop-bottle {
    opacity: 1;
    animation: bottle-wiggle 1.8s ease-in-out infinite;
  }
  @keyframes bottle-wiggle {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-6deg);
    }
  }

  .state-happy .eye-white {
    display: none;
  }
  .state-happy .pupil {
    display: none;
  }
  .state-happy .eye .eyelid {
    opacity: 1;
    stroke-width: 4;
    transform: translateY(-4px) scaleY(1.4);
  }
  .state-happy .body-svg {
    animation: happy-jump 0.5s ease-in-out 2;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes happy-jump {
    0%,
    100% {
      transform: translateY(0);
    }
    40% {
      transform: translateY(-18px);
    }
  }
  @keyframes sparkle-burst {
    0% {
      opacity: 0;
      transform: scale(0.3);
    }
    40% {
      opacity: 1;
      transform: scale(1.1);
    }
    100% {
      opacity: 0;
      transform: scale(1.4);
    }
  }

  .state-shrug .arm {
    animation: shrug-arms 600ms ease-out both;
  }
  @keyframes shrug-arms {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(-14px);
    }
  }
  .state-shrug .brow {
    transform: translateY(2px);
  }

  .state-sad .brow {
    transform: translateY(4px) rotate(4deg);
  }
  .state-sad .tear {
    opacity: 1;
    animation: tear-drop 1.4s ease-in infinite;
  }
  @keyframes tear-drop {
    0% {
      opacity: 0;
      transform: translateY(0);
    }
    20% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: translateY(18px);
    }
  }

  .state-wave .arm-right {
    animation: wave 500ms ease-in-out 3;
    transform-origin: 162px 150px;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-30deg);
    }
  }

  /* ---------- Walk enter/exit ---------- */

  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.5s ease-in-out infinite,
      hop-walk 420ms ease-in-out infinite;
  }
  @keyframes hop-walk {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-10px) rotate(-3deg);
    }
  }

  .dust-puffs {
    position: absolute;
    bottom: 4px;
    left: 50%;
    translate: -50% 0;
    width: 80px;
    height: 20px;
    pointer-events: none;
  }
  .puff {
    position: absolute;
    bottom: 0;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgba(200, 180, 150, 0.5);
    animation: puff-rise 420ms ease-out infinite;
  }
  .p1 {
    left: 10px;
    animation-delay: 0ms;
  }
  .p2 {
    left: 35px;
    animation-delay: 140ms;
  }
  .p3 {
    left: 60px;
    animation-delay: 280ms;
  }
  @keyframes puff-rise {
    0% {
      opacity: 0.6;
      transform: translateY(0) scale(0.6);
    }
    100% {
      opacity: 0;
      transform: translateY(-16px) scale(1.2);
    }
  }
</style>
