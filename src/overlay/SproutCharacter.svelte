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
    const cy = rect.top + rect.height * 0.5;
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
    const delay = 2600 + Math.random() * 3600;
    blinkTimer = setTimeout(async () => {
      blinking = true;
      await new Promise((r) => setTimeout(r, 120));
      blinking = false;
      if (Math.random() < 0.2) {
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
    happy: "M 80 148 Q 100 168 120 148",
    shrug: "M 88 152 Q 100 150 112 152",
    sad: "M 85 154 Q 100 146 115 154",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 86 148 Q 100 156 114 148");
</script>

<div class="sprout state-{sceneState}" style="--scale: {scale}" bind:this={rootEl}>
  {#if sceneState === "walk_in" || sceneState === "walk_out"}
    <div class="dust-puffs">
      <span class="puff p1"></span>
      <span class="puff p2"></span>
      <span class="puff p3"></span>
    </div>
  {/if}

  <svg class="body-svg" viewBox="0 0 200 230" width="140" height="161" aria-hidden="true">
    <!-- Back leaves (behind head) -->
    <path class="leaf leaf-back-left" d="M 70 60 Q 20 50 15 90 Q 50 95 70 68 Z" />
    <path class="leaf leaf-back-right" d="M 130 60 Q 180 50 185 90 Q 150 95 130 68 Z" />

    <!-- Pot -->
    <path class="pot" d="M 55 165 L 145 165 L 135 215 L 65 215 Z" />
    <rect class="pot-rim" x="50" y="155" width="100" height="16" rx="6" />
    <path class="pot-shine" d="M 68 172 L 75 205" />

    <!-- Stem -->
    <rect class="stem" x="94" y="110" width="12" height="55" rx="5" />

    <!-- Head / main leaf cluster (round face) -->
    <circle class="face" cx="100" cy="95" r="50" />

    <!-- Top leaves -->
    <path class="leaf leaf-top-left" d="M 85 50 Q 55 20 70 -5 Q 100 10 95 48 Z" />
    <path class="leaf leaf-top-right" d="M 115 50 Q 145 20 130 -5 Q 100 10 105 48 Z" />
    <path class="leaf leaf-top-center" d="M 100 45 Q 95 5 100 -15 Q 105 5 100 45 Z" />

    <!-- Arms (little leaf-stub arms) -->
    <ellipse class="arm arm-left" cx="48" cy="105" rx="13" ry="9" />
    <ellipse class="arm arm-right" cx="152" cy="105" rx="13" ry="9" />

    <!-- Blush -->
    <ellipse class="blush blush-left" cx="75" cy="100" rx="9" ry="6" />
    <ellipse class="blush blush-right" cx="125" cy="100" rx="9" ry="6" />

    <!-- Eyes -->
    <g class="eye eye-left" class:closed={eyesClosed}>
      <circle class="eye-white" cx="82" cy="90" r="10" />
      <circle class="pupil" cx={82 + pupilOffset.x} cy={90 + pupilOffset.y} r="4.5" />
      <circle class="eye-shine" cx={82 + pupilOffset.x - 1.4} cy={90 + pupilOffset.y - 1.8} r="1.4" />
      <path class="eyelid" d="M 70 90 Q 82 90 94 90" />
    </g>
    <g class="eye eye-right" class:closed={eyesClosed}>
      <circle class="eye-white" cx="118" cy="90" r="10" />
      <circle class="pupil" cx={118 + pupilOffset.x} cy={90 + pupilOffset.y} r="4.5" />
      <circle class="eye-shine" cx={118 + pupilOffset.x - 1.4} cy={90 + pupilOffset.y - 1.8} r="1.4" />
      <path class="eyelid" d="M 106 90 Q 118 90 130 90" />
    </g>

    <!-- Eyebrows -->
    <path class="brow brow-left" d="M 72 76 Q 82 70 92 76" />
    <path class="brow brow-right" d="M 108 76 Q 118 70 128 76" />

    <!-- Mouth -->
    <path class="mouth" d={mouthPath} />

    <!-- Tear for sad -->
    <ellipse class="tear" cx="92" cy="97" rx="3" ry="4.5" />

    <!-- Watering-can prop for ask -->
    <g class="prop-can">
      <rect x="150" y="95" width="24" height="18" rx="4" />
      <path d="M 174 100 L 190 92" stroke-width="5" />
      <circle cx="191" cy="91" r="3" />
      <rect x="156" y="86" width="8" height="10" rx="2" />
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
  .sprout {
    --leaf: #6fcf7a;
    --leaf-dark: #3fa650;
    --pot-color: #d9824a;
    --pot-dark: #b8632f;
    --blush: #ff9ec7;

    position: relative;
    width: calc(140px * var(--scale));
    height: calc(161px * var(--scale));
    transform-origin: bottom center;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    animation: breathe 2.7s ease-in-out infinite;
    transform-origin: 50% 100%;
  }

  @keyframes breathe {
    0%,
    100% {
      transform: scaleX(1) scaleY(1);
    }
    50% {
      transform: scaleX(1.025) scaleY(0.975);
    }
  }

  .face,
  .arm {
    fill: var(--leaf);
    stroke: var(--leaf-dark);
    stroke-width: 2;
  }

  .leaf {
    fill: var(--leaf);
    stroke: var(--leaf-dark);
    stroke-width: 2;
  }

  .stem {
    fill: var(--leaf-dark);
  }

  .pot {
    fill: var(--pot-color);
    stroke: var(--pot-dark);
    stroke-width: 2;
  }
  .pot-rim {
    fill: var(--pot-dark);
  }
  .pot-shine {
    stroke: rgba(255, 255, 255, 0.4);
    stroke-width: 4;
    stroke-linecap: round;
  }

  .blush {
    fill: var(--blush);
    opacity: 0.5;
  }

  .eye-white {
    fill: white;
    stroke: var(--leaf-dark);
    stroke-width: 1.4;
  }

  .pupil {
    fill: #233318;
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
    stroke: var(--leaf-dark);
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
    stroke: var(--leaf-dark);
    stroke-width: 3;
    stroke-linecap: round;
    transition: transform 220ms ease;
  }

  .mouth {
    fill: none;
    stroke: #234a28;
    stroke-width: 3;
    stroke-linecap: round;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-can {
    opacity: 0;
    transform-origin: 160px 100px;
  }
  .prop-can rect,
  .prop-can circle {
    fill: #9ec7e8;
    stroke: #5a8fb8;
    stroke-width: 1.5;
  }
  .prop-can path {
    stroke: #5a8fb8;
    fill: none;
    stroke-linecap: round;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(30px, 55px);
  }
  .s2 {
    transform: translate(165px, 70px);
  }
  .s3 {
    transform: translate(100px, 15px);
  }

  /* States */
  .state-ask .leaf-top-center {
    animation: leaf-sway 1.8s ease-in-out infinite;
    transform-origin: 100px 45px;
  }
  .state-ask .prop-can {
    opacity: 1;
    animation: can-tilt 1.6s ease-in-out infinite;
  }
  @keyframes leaf-sway {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-6deg);
    }
  }
  @keyframes can-tilt {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-10deg);
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
      transform: translateY(-14px);
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
      transform: translateY(-11px);
    }
  }

  .state-sad .leaf-top-left,
  .state-sad .leaf-top-right {
    transform-origin: 100px 45px;
    transform: rotate(8deg) translateY(4px);
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
    transform-origin: 152px 105px;
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

  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.7s ease-in-out infinite,
      hop-walk 420ms ease-in-out infinite;
  }
  @keyframes hop-walk {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-8px) rotate(-2deg);
    }
  }

  .dust-puffs {
    position: absolute;
    bottom: 2px;
    left: 50%;
    translate: -50% 0;
    width: 70px;
    height: 18px;
    pointer-events: none;
  }
  .puff {
    position: absolute;
    bottom: 0;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgba(200, 180, 150, 0.5);
    animation: puff-rise 400ms ease-out infinite;
  }
  .p1 {
    left: 8px;
    animation-delay: 0ms;
  }
  .p2 {
    left: 30px;
    animation-delay: 130ms;
  }
  .p3 {
    left: 52px;
    animation-delay: 260ms;
  }
  @keyframes puff-rise {
    0% {
      opacity: 0.6;
      transform: translateY(0) scale(0.6);
    }
    100% {
      opacity: 0;
      transform: translateY(-14px) scale(1.2);
    }
  }
</style>
