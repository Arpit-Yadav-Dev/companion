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
    const cy = rect.top + rect.height * 0.32;
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
    const delay = 2200 + Math.random() * 3800;
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
    happy: "M 75 150 Q 100 172 125 150",
    shrug: "M 85 154 Q 100 152 115 154",
    sad: "M 82 156 Q 100 148 118 156",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 82 150 Q 100 160 118 150");
</script>

<div class="biscuit state-{sceneState}" style="--scale: {scale}" bind:this={rootEl}>
  {#if sceneState === "walk_in" || sceneState === "walk_out"}
    <div class="dust-puffs">
      <span class="puff p1"></span>
      <span class="puff p2"></span>
      <span class="puff p3"></span>
    </div>
  {/if}

  <svg class="body-svg" viewBox="0 0 200 210" width="150" height="158" aria-hidden="true">
    <!-- Tail -->
    <path class="tail" d="M 160 150 Q 185 135 180 105" />

    <!-- Ears (floppy, droopy) -->
    <path class="ear ear-left" d="M 55 95 Q 30 110 35 150 Q 50 145 58 118 Z" />
    <path class="ear ear-right" d="M 145 95 Q 170 110 165 150 Q 150 145 142 118 Z" />
    <path class="ear-fold ear-fold-left" d="M 48 108 Q 42 125 44 142" />
    <path class="ear-fold ear-fold-right" d="M 152 108 Q 158 125 156 142" />

    <!-- Body -->
    <ellipse class="body" cx="100" cy="150" rx="52" ry="45" />
    <!-- Head -->
    <circle class="body" cx="100" cy="110" r="48" />
    <!-- Snout -->
    <ellipse class="snout" cx="100" cy="135" rx="26" ry="18" />

    <!-- Feet -->
    <ellipse class="foot foot-left" cx="75" cy="193" rx="15" ry="9" />
    <ellipse class="foot foot-right" cx="125" cy="193" rx="15" ry="9" />

    <!-- Paws / arms -->
    <ellipse class="arm arm-left" cx="48" cy="155" rx="12" ry="10" />
    <ellipse class="arm arm-right" cx="152" cy="155" rx="12" ry="10" />

    <!-- Chest patch -->
    <ellipse class="patch" cx="100" cy="158" rx="22" ry="20" />

    <!-- Blush -->
    <ellipse class="blush blush-left" cx="68" cy="122" rx="9" ry="6" />
    <ellipse class="blush blush-right" cx="132" cy="122" rx="9" ry="6" />

    <!-- Eyes -->
    <g class="eye eye-left" class:closed={eyesClosed}>
      <circle class="eye-white" cx="80" cy="105" r="11" />
      <circle class="pupil" cx={80 + pupilOffset.x} cy={105 + pupilOffset.y} r="5" />
      <circle class="eye-shine" cx={80 + pupilOffset.x - 1.5} cy={105 + pupilOffset.y - 2} r="1.6" />
      <path class="eyelid" d="M 68 105 Q 80 105 92 105" />
    </g>
    <g class="eye eye-right" class:closed={eyesClosed}>
      <circle class="eye-white" cx="120" cy="105" r="11" />
      <circle class="pupil" cx={120 + pupilOffset.x} cy={105 + pupilOffset.y} r="5" />
      <circle class="eye-shine" cx={120 + pupilOffset.x - 1.5} cy={105 + pupilOffset.y - 2} r="1.6" />
      <path class="eyelid" d="M 108 105 Q 120 105 132 105" />
    </g>

    <!-- Eyebrows -->
    <path class="brow brow-left" d="M 70 88 Q 80 82 90 88" />
    <path class="brow brow-right" d="M 110 88 Q 120 82 130 88" />

    <!-- Nose -->
    <ellipse class="nose" cx="100" cy="130" rx="7" ry="5" />

    <!-- Mouth -->
    <path class="mouth" d={mouthPath} />

    <!-- Tongue for happy -->
    <path class="tongue" d="M 95 152 Q 100 165 105 152 Z" />

    <!-- Tear for sad -->
    <ellipse class="tear" cx="90" cy="112" rx="3" ry="5" />

    <!-- Water-bottle-style prop for ask -->
    <g class="prop-bone">
      <rect x="158" y="128" width="20" height="8" rx="4" />
      <circle cx="158" cy="126" r="5" />
      <circle cx="158" cy="134" r="5" />
      <circle cx="178" cy="126" r="5" />
      <circle cx="178" cy="134" r="5" />
    </g>

    <!-- Sparkles for happy -->
    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .biscuit {
    --scale: 1;
    --fur: #d9a066;
    --fur-dark: #c08848;
    --snout: #f0dcc0;
    --blush: #ff9ec7;

    position: relative;
    width: calc(150px * var(--scale));
    height: calc(158px * var(--scale));
    transform-origin: bottom center;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    animation: breathe 2.6s ease-in-out infinite;
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

  .body,
  .foot,
  .arm {
    fill: var(--fur);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }

  .ear {
    fill: var(--fur-dark);
  }

  .snout,
  .patch {
    fill: var(--snout);
  }

  .tail {
    fill: none;
    stroke: var(--fur);
    stroke-width: 16;
    stroke-linecap: round;
  }

  .nose {
    fill: #3a2a1a;
  }

  .blush {
    fill: var(--blush);
    opacity: 0.55;
  }

  .eye-white {
    fill: white;
    stroke: var(--fur-dark);
    stroke-width: 1.5;
  }

  .pupil {
    fill: #2a1f14;
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

  .ear-fold {
    fill: none;
    stroke: rgba(0, 0, 0, 0.15);
    stroke-width: 2;
    stroke-linecap: round;
  }

  .eyelid {
    fill: none;
    stroke: var(--fur-dark);
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
    stroke: var(--fur-dark);
    stroke-width: 3.5;
    stroke-linecap: round;
    transition: transform 220ms ease;
  }

  .mouth {
    fill: none;
    stroke: #6b4a2a;
    stroke-width: 3.5;
    stroke-linecap: round;
  }

  .tongue {
    fill: #ff8fa3;
    opacity: 0;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-bone {
    opacity: 0;
    transform-origin: 168px 130px;
  }
  .prop-bone rect,
  .prop-bone circle {
    fill: #f5f1e8;
    stroke: var(--fur-dark);
    stroke-width: 1.5;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(40px, 70px);
  }
  .s2 {
    transform: translate(160px, 90px);
  }

  /* States */
  .state-ask .ear-left {
    animation: ear-perk 1.8s ease-in-out infinite;
  }
  .state-ask .prop-bone {
    opacity: 1;
    animation: bone-wiggle 1.6s ease-in-out infinite;
  }
  @keyframes ear-perk {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-6deg);
    }
  }
  @keyframes bone-wiggle {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-8deg);
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
  .state-happy .tongue {
    opacity: 1;
  }
  .state-happy .body-svg {
    animation: happy-wag 0.4s ease-in-out infinite;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes happy-wag {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(4deg);
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
      transform: translateY(-12px);
    }
  }

  .state-sad .ear-left,
  .state-sad .ear-right {
    transform: translateY(6px);
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
      transform: translateY(16px);
    }
  }

  .state-wave .arm-right {
    animation: wave 500ms ease-in-out 3;
    transform-origin: 152px 155px;
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

  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.6s ease-in-out infinite,
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
    bottom: 3px;
    left: 50%;
    translate: -50% 0;
    width: 75px;
    height: 20px;
    pointer-events: none;
  }
  .puff {
    position: absolute;
    bottom: 0;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: rgba(200, 180, 150, 0.5);
    animation: puff-rise 410ms ease-out infinite;
  }
  .p1 {
    left: 10px;
    animation-delay: 0ms;
  }
  .p2 {
    left: 32px;
    animation-delay: 135ms;
  }
  .p3 {
    left: 55px;
    animation-delay: 270ms;
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
