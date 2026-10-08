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
    scale?: number;
    mirrored?: boolean;
  }

  let { sceneState, scale = 1, mirrored = false }: Props = $props();

  let rootEl: HTMLDivElement | undefined = $state();
  let pupilOffset = $state({ x: 0, y: 0 });

  function onCursorPosition(pos: CursorPositionPayload) {
    if (!rootEl) return;
    const rect = rootEl.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height * 0.4;
    const dx = (pos.x - cx) * (mirrored ? -1 : 1);
    const dy = pos.y - cy;
    const dist = Math.hypot(dx, dy) || 1;
    const maxOffset = 3;
    pupilOffset = {
      x: (dx / dist) * Math.min(maxOffset, dist / 40),
      y: (dy / dist) * Math.min(maxOffset, dist / 40),
    };
  }

  let blinking = $state(false);
  let blinkTimer: ReturnType<typeof setTimeout> | undefined;

  function scheduleBlink() {
    const delay = 2400 + Math.random() * 3600;
    blinkTimer = setTimeout(async () => {
      blinking = true;
      await new Promise((r) => setTimeout(r, 120));
      blinking = false;
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
    happy: "M 78 118 Q 100 140 122 118",
    shrug: "M 86 122 Q 100 120 114 122",
    sad: "M 83 124 Q 100 116 117 124",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 84 118 Q 100 128 116 118");
</script>

<div class="marshmallow state-{sceneState}" style="--scale: {scale}" bind:this={rootEl}>
  {#if sceneState === "walk_in" || sceneState === "walk_out"}
    <div class="sparkle-trail">
      <span class="trail-dot t1"></span>
      <span class="trail-dot t2"></span>
      <span class="trail-dot t3"></span>
    </div>
  {/if}

  <svg class="body-svg" viewBox="0 0 200 200" width="150" height="150" aria-hidden="true">
    <!-- Floaty blob body with scalloped bottom (ghost-like) -->
    <path
      class="body"
      d="M 100 20
         C 150 20, 170 60, 170 100
         C 170 130, 170 150, 170 160
         Q 158 175, 146 160
         Q 134 175, 122 160
         Q 110 175, 98 160
         Q 86 175, 74 160
         Q 62 175, 50 160
         C 30 150, 30 130, 30 100
         C 30 60, 50 20, 100 20
         Z"
    />

    <!-- Glossy highlight -->
    <ellipse class="highlight" cx="72" cy="62" rx="16" ry="22" />

    <!-- Stubby arms -->
    <ellipse class="arm arm-left" cx="30" cy="105" rx="13" ry="10" />
    <ellipse class="arm arm-right" cx="170" cy="105" rx="13" ry="10" />

    <!-- Blush -->
    <ellipse class="blush blush-left" cx="70" cy="108" rx="10" ry="6" />
    <ellipse class="blush blush-right" cx="130" cy="108" rx="10" ry="6" />

    <!-- Eyes -->
    <g class="eye eye-left" class:closed={eyesClosed}>
      <circle class="eye-white" cx="78" cy="90" r="11" />
      <circle class="pupil" cx={78 + pupilOffset.x} cy={90 + pupilOffset.y} r="5" />
      <circle class="eye-shine" cx={78 + pupilOffset.x - 1.5} cy={90 + pupilOffset.y - 2} r="1.6" />
      <path class="eyelid" d="M 66 90 Q 78 90 90 90" />
    </g>
    <g class="eye eye-right" class:closed={eyesClosed}>
      <circle class="eye-white" cx="122" cy="90" r="11" />
      <circle class="pupil" cx={122 + pupilOffset.x} cy={90 + pupilOffset.y} r="5" />
      <circle class="eye-shine" cx={122 + pupilOffset.x - 1.5} cy={90 + pupilOffset.y - 2} r="1.6" />
      <path class="eyelid" d="M 110 90 Q 122 90 134 90" />
    </g>

    <!-- Eyebrows -->
    <path class="brow brow-left" d="M 68 73 Q 78 67 88 73" />
    <path class="brow brow-right" d="M 112 73 Q 122 67 132 73" />

    <!-- Mouth -->
    <path class="mouth" d={mouthPath} />

    <!-- Tear for sad -->
    <ellipse class="tear" cx="88" cy="97" rx="3" ry="4.5" />

    <!-- Tiny floating star prop for ask -->
    <g class="prop-star">
      <path d="M0 -10 L3 -3 L10 0 L3 3 L0 10 L-3 3 L-10 0 L-3 -3 Z" />
    </g>

    <!-- Sparkles for happy -->
    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s3" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .marshmallow {
    --body-color: #f3e9ff;
    --body-dark: #c9a9f0;
    --blush: #ffb3d9;

    position: relative;
    width: calc(150px * var(--scale));
    height: calc(150px * var(--scale));
    transform-origin: bottom center;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    animation: float 3s ease-in-out infinite;
    transform-origin: 50% 100%;
  }

  @keyframes float {
    0%,
    100% {
      transform: translateY(0) scaleX(1) scaleY(1);
    }
    50% {
      transform: translateY(-6px) scaleX(1.02) scaleY(0.98);
    }
  }

  .body,
  .arm {
    fill: var(--body-color);
    stroke: var(--body-dark);
    stroke-width: 2.5;
  }

  .highlight {
    fill: white;
    opacity: 0.6;
  }

  .blush {
    fill: var(--blush);
    opacity: 0.6;
  }

  .eye-white {
    fill: white;
    stroke: var(--body-dark);
    stroke-width: 1.5;
  }

  .pupil {
    fill: #4a3a5a;
    transition:
      cx 80ms linear,
      cy 80ms linear;
  }

  .eye-shine {
    fill: white;
    opacity: 0.9;
    transition:
      cx 80ms linear,
      cy 80ms linear;
  }

  .eyelid {
    fill: none;
    stroke: var(--body-dark);
    stroke-width: 3;
    stroke-linecap: round;
    opacity: 0;
  }

  .eye.closed .eye-white,
  .eye.closed .pupil,
  .eye.closed .eye-shine {
    opacity: 0;
  }
  .eye.closed .eyelid {
    opacity: 1;
  }

  .brow {
    fill: none;
    stroke: var(--body-dark);
    stroke-width: 3.5;
    stroke-linecap: round;
    transition: transform 220ms ease;
  }

  .mouth {
    fill: none;
    stroke: #6a4a80;
    stroke-width: 3.5;
    stroke-linecap: round;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-star {
    opacity: 0;
    fill: #ffe14d;
    transform: translate(165px, 70px);
    transform-origin: 165px 70px;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(30px, 40px);
  }
  .s2 {
    transform: translate(170px, 55px);
  }
  .s3 {
    transform: translate(100px, 10px);
  }

  /* States */
  .state-ask .prop-star {
    opacity: 1;
    animation: star-twinkle 1.4s ease-in-out infinite;
  }
  @keyframes star-twinkle {
    0%,
    100% {
      transform: translate(165px, 70px) scale(1) rotate(0deg);
    }
    50% {
      transform: translate(165px, 64px) scale(1.2) rotate(20deg);
    }
  }

  .state-happy .eye .eyelid {
    opacity: 1;
    transform: translateY(-3px) scaleY(1.3);
  }
  .state-happy .eye-white,
  .state-happy .pupil,
  .state-happy .eye-shine {
    display: none;
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
      transform: translateY(-16px);
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
    animation: shrug-arms 550ms ease-out both;
  }
  @keyframes shrug-arms {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(-12px);
    }
  }

  .state-sad .tear {
    opacity: 1;
    animation: tear-drop 1.3s ease-in infinite;
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
      transform: translateY(14px);
    }
  }

  .state-wave .arm-right {
    animation: wave 480ms ease-in-out 3;
    transform-origin: 170px 105px;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-28deg);
    }
  }

  /* Marshmallow floats in/out instead of walking — no feet to animate a
     walk cycle with, so lean into the "ghost" identity instead. */
  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      float 3s ease-in-out infinite,
      bob-in 400ms ease-in-out infinite;
  }
  @keyframes bob-in {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-10px) rotate(-2deg);
    }
  }

  .sparkle-trail {
    position: absolute;
    bottom: 20px;
    left: 50%;
    translate: -50% 0;
    width: 90px;
    height: 24px;
    pointer-events: none;
  }
  .trail-dot {
    position: absolute;
    bottom: 0;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: rgba(201, 169, 240, 0.6);
    animation: trail-fade 450ms ease-out infinite;
  }
  .t1 {
    left: 10px;
    animation-delay: 0ms;
  }
  .t2 {
    left: 40px;
    animation-delay: 150ms;
  }
  .t3 {
    left: 70px;
    animation-delay: 300ms;
  }
  @keyframes trail-fade {
    0% {
      opacity: 0.7;
      transform: scale(1);
    }
    100% {
      opacity: 0;
      transform: scale(1.8);
    }
  }
</style>
