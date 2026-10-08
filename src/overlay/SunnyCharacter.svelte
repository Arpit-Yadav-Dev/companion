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
    const cy = rect.top + rect.height * 0.45;
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
    const delay = 2300 + Math.random() * 3700;
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
    happy: "M 78 128 Q 100 152 122 128",
    shrug: "M 86 132 Q 100 130 114 132",
    sad: "M 83 134 Q 100 126 117 134",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 84 128 Q 100 138 116 128");
</script>

<div class="sunny state-{sceneState}" style="--scale: {scale}" bind:this={rootEl}>
  {#if sceneState === "walk_in" || sceneState === "walk_out"}
    <div class="dust-puffs">
      <span class="puff p1"></span>
      <span class="puff p2"></span>
      <span class="puff p3"></span>
    </div>
  {/if}

  <svg class="body-svg" viewBox="0 0 200 200" width="150" height="150" aria-hidden="true">
    <!-- Rays -->
    <g class="rays">
      <path d="M 100 20 L 108 0 L 92 0 Z" />
      <path d="M 100 180 L 108 200 L 92 200 Z" />
      <path d="M 20 100 L 0 92 L 0 108 Z" />
      <path d="M 180 100 L 200 92 L 200 108 Z" />
      <path d="M 44 44 L 26 26 L 36 18 Z" />
      <path d="M 156 44 L 174 26 L 164 18 Z" />
      <path d="M 44 156 L 26 174 L 36 182 Z" />
      <path d="M 156 156 L 174 174 L 164 182 Z" />
    </g>

    <!-- Core body -->
    <circle class="core" cx="100" cy="100" r="68" />
    <!-- Gradient highlight -->
    <ellipse class="highlight" cx="78" cy="78" rx="18" ry="24" />

    <!-- Arms -->
    <ellipse class="arm arm-left" cx="34" cy="108" rx="13" ry="10" />
    <ellipse class="arm arm-right" cx="166" cy="108" rx="13" ry="10" />

    <!-- Feet -->
    <ellipse class="foot foot-left" cx="80" cy="162" rx="14" ry="8" />
    <ellipse class="foot foot-right" cx="120" cy="162" rx="14" ry="8" />

    <!-- Blush -->
    <ellipse class="blush blush-left" cx="68" cy="112" rx="10" ry="6" />
    <ellipse class="blush blush-right" cx="132" cy="112" rx="10" ry="6" />

    <!-- Eyes -->
    <g class="eye eye-left" class:closed={eyesClosed}>
      <circle class="eye-white" cx="78" cy="95" r="11" />
      <circle class="pupil" cx={78 + pupilOffset.x} cy={95 + pupilOffset.y} r="5" />
      <circle class="eye-shine" cx={78 + pupilOffset.x - 1.5} cy={95 + pupilOffset.y - 2} r="1.6" />
      <path class="eyelid" d="M 66 95 Q 78 95 90 95" />
    </g>
    <g class="eye eye-right" class:closed={eyesClosed}>
      <circle class="eye-white" cx="122" cy="95" r="11" />
      <circle class="pupil" cx={122 + pupilOffset.x} cy={95 + pupilOffset.y} r="5" />
      <circle class="eye-shine" cx={122 + pupilOffset.x - 1.5} cy={95 + pupilOffset.y - 2} r="1.6" />
      <path class="eyelid" d="M 110 95 Q 122 95 134 95" />
    </g>

    <!-- Eyebrows -->
    <path class="brow brow-left" d="M 68 78 Q 78 72 88 78" />
    <path class="brow brow-right" d="M 112 78 Q 122 72 132 78" />

    <!-- Mouth -->
    <path class="mouth" d={mouthPath} />

    <!-- Tear for sad -->
    <ellipse class="tear" cx="88" cy="102" rx="3" ry="4.5" />

    <!-- Sunglasses prop for ask (silly, charming) -->
    <g class="prop-glasses">
      <rect x="64" y="86" width="28" height="16" rx="6" />
      <rect x="108" y="86" width="28" height="16" rx="6" />
      <rect x="92" y="92" width="16" height="4" />
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
  .sunny {
    --core: #ffd34d;
    --core-dark: #f4a623;
    --ray: #ffcf3d;
    --blush: #ff9ec7;

    position: relative;
    width: calc(150px * var(--scale));
    height: calc(150px * var(--scale));
    transform-origin: bottom center;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    animation: breathe 2.4s ease-in-out infinite;
    transform-origin: 50% 65%;
  }

  @keyframes breathe {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.035);
    }
  }

  .rays {
    animation: ray-spin 16s linear infinite;
    transform-origin: 100px 100px;
  }
  .rays path {
    fill: var(--ray);
  }
  @keyframes ray-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .core,
  .arm,
  .foot {
    fill: var(--core);
    stroke: var(--core-dark);
    stroke-width: 2;
  }

  .highlight {
    fill: white;
    opacity: 0.4;
  }

  .blush {
    fill: var(--blush);
    opacity: 0.55;
  }

  .eye-white {
    fill: white;
    stroke: var(--core-dark);
    stroke-width: 1.5;
  }

  .pupil {
    fill: #4a3410;
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
    stroke: var(--core-dark);
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
    stroke: var(--core-dark);
    stroke-width: 3.5;
    stroke-linecap: round;
    transition: transform 220ms ease;
  }

  .mouth {
    fill: none;
    stroke: #7a4a10;
    stroke-width: 3.5;
    stroke-linecap: round;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-glasses {
    opacity: 0;
    fill: #2a2a2a;
    transform-origin: 100px 94px;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #fff3b0;
  }
  .s1 {
    transform: translate(30px, 40px);
  }
  .s2 {
    transform: translate(170px, 60px);
  }
  .s3 {
    transform: translate(100px, 10px);
  }

  /* States */
  .state-ask .prop-glasses {
    opacity: 1;
    animation: glasses-cool 1.8s ease-in-out infinite;
  }
  @keyframes glasses-cool {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-2px) rotate(-3deg);
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
  .state-happy .rays {
    animation:
      ray-spin 16s linear infinite,
      ray-glow 0.5s ease-in-out 2;
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
  @keyframes ray-glow {
    0%,
    100% {
      transform: scale(1) rotate(0deg);
    }
    50% {
      transform: scale(1.15) rotate(8deg);
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

  .state-sad .rays {
    animation-play-state: paused;
    opacity: 0.6;
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
    transform-origin: 166px 108px;
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
      breathe 2.4s ease-in-out infinite,
      hop-walk 400ms ease-in-out infinite;
  }
  @keyframes hop-walk {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-9px) rotate(-3deg);
    }
  }

  .dust-puffs {
    position: absolute;
    bottom: 2px;
    left: 50%;
    translate: -50% 0;
    width: 75px;
    height: 18px;
    pointer-events: none;
  }
  .puff {
    position: absolute;
    bottom: 0;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: rgba(200, 180, 150, 0.5);
    animation: puff-rise 400ms ease-out infinite;
  }
  .p1 {
    left: 10px;
    animation-delay: 0ms;
  }
  .p2 {
    left: 32px;
    animation-delay: 130ms;
  }
  .p3 {
    left: 55px;
    animation-delay: 260ms;
  }
  @keyframes puff-rise {
    0% {
      opacity: 0.6;
      transform: translateY(0) scale(0.6);
    }
    100% {
      opacity: 0;
      transform: translateY(-15px) scale(1.2);
    }
  }
</style>
