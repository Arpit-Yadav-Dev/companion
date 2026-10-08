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
    const cy = rect.top + rect.height * 0.3;
    const dx = (pos.x - cx) * (mirrored ? -1 : 1);
    const dy = pos.y - cy;
    const dist = Math.hypot(dx, dy) || 1;
    const maxOffset = 2.5;
    pupilOffset = {
      x: (dx / dist) * Math.min(maxOffset, dist / 40),
      y: (dy / dist) * Math.min(maxOffset, dist / 40),
    };
  }

  let blinking = $state(false);
  let blinkTimer: ReturnType<typeof setTimeout> | undefined;

  function scheduleBlink() {
    const delay = 2500 + Math.random() * 3500;
    blinkTimer = setTimeout(async () => {
      blinking = true;
      await new Promise((r) => setTimeout(r, 110));
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
    happy: "M 78 68 Q 90 78 102 68",
    shrug: "M 82 70 Q 90 69 98 70",
    sad: "M 80 72 Q 90 66 100 72",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 80 68 Q 90 74 100 68");
</script>

<div class="pixel state-{sceneState}" style="--scale: {scale}" bind:this={rootEl}>
  {#if sceneState === "walk_in" || sceneState === "walk_out"}
    <div class="dust-puffs">
      <span class="puff p1"></span>
      <span class="puff p2"></span>
      <span class="puff p3"></span>
    </div>
  {/if}

  <svg class="body-svg" viewBox="0 0 180 170" width="140" height="132" aria-hidden="true">
    <!-- Tail -->
    <path class="tail" d="M 150 120 Q 175 100 165 70" />

    <!-- Ears -->
    <path class="ear ear-left" d="M 55 30 L 45 5 L 75 25 Z" />
    <path class="ear ear-right" d="M 125 30 L 135 5 L 105 25 Z" />

    <!-- Body (flat blob) -->
    <ellipse class="body" cx="90" cy="100" rx="55" ry="48" />
    <!-- Head -->
    <circle class="body" cx="90" cy="55" r="42" />

    <!-- Feet -->
    <ellipse class="foot foot-left" cx="65" cy="145" rx="14" ry="8" />
    <ellipse class="foot foot-right" cx="115" cy="145" rx="14" ry="8" />

    <!-- Paws / arms -->
    <ellipse class="arm arm-left" cx="42" cy="105" rx="11" ry="9" />
    <ellipse class="arm arm-right" cx="138" cy="105" rx="11" ry="9" />

    <!-- Belly patch (2nd flat colour) -->
    <ellipse class="belly" cx="90" cy="108" rx="28" ry="26" />

    <!-- Blush -->
    <ellipse class="blush blush-left" cx="62" cy="62" rx="8" ry="5" />
    <ellipse class="blush blush-right" cx="118" cy="62" rx="8" ry="5" />

    <!-- Eyes -->
    <g class="eye eye-left" class:closed={eyesClosed}>
      <circle class="eye-white" cx="72" cy="52" r="10" />
      <circle class="pupil" cx={72 + pupilOffset.x} cy={52 + pupilOffset.y} r="4.5" />
      <path class="eyelid" d="M 62 52 Q 72 52 82 52" />
    </g>
    <g class="eye eye-right" class:closed={eyesClosed}>
      <circle class="eye-white" cx="108" cy="52" r="10" />
      <circle class="pupil" cx={108 + pupilOffset.x} cy={52 + pupilOffset.y} r="4.5" />
      <path class="eyelid" d="M 98 52 Q 108 52 118 52" />
    </g>

    <!-- Whiskers -->
    <g class="whiskers">
      <path d="M 40 58 L 15 54" />
      <path d="M 40 64 L 15 66" />
      <path d="M 140 58 L 165 54" />
      <path d="M 140 64 L 165 66" />
    </g>

    <!-- Mouth -->
    <path class="mouth" d={mouthPath} />

    <!-- Tear for sad -->
    <ellipse class="tear" cx="80" cy="64" rx="2.5" ry="4" />

    <!-- Sparkles for happy -->
    <g class="sparkles">
      <path class="sparkle s1" d="M0 -7 L1.5 -1.5 L7 0 L1.5 1.5 L0 7 L-1.5 1.5 L-7 0 L-1.5 -1.5 Z" />
      <path class="sparkle s2" d="M0 -7 L1.5 -1.5 L7 0 L1.5 1.5 L0 7 L-1.5 1.5 L-7 0 L-1.5 -1.5 Z" />
    </g>
  </svg>
</div>

<style>
  .pixel {
    --scale: 1;
    --fur: #3a3a3a;
    --fur-belly: #f2f2f2;
    --blush: #ff9ec7;

    position: relative;
    width: calc(140px * var(--scale));
    height: calc(132px * var(--scale));
    transform-origin: bottom center;
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
      transform: scaleX(1.02) scaleY(0.98);
    }
  }

  .body,
  .ear,
  .foot,
  .arm {
    fill: var(--fur);
  }

  .belly {
    fill: var(--fur-belly);
  }

  .tail {
    fill: none;
    stroke: var(--fur);
    stroke-width: 14;
    stroke-linecap: round;
  }

  .blush {
    fill: var(--blush);
    opacity: 0.5;
  }

  .eye-white {
    fill: white;
  }

  .pupil {
    fill: #111;
    transition:
      cx 80ms linear,
      cy 80ms linear;
  }

  .eyelid {
    fill: none;
    stroke: var(--fur);
    stroke-width: 2.5;
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

  .whiskers path {
    stroke: var(--fur-belly);
    stroke-width: 2;
    stroke-linecap: round;
  }

  .mouth {
    fill: none;
    stroke: #222;
    stroke-width: 2.5;
    stroke-linecap: round;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(35px, 50px);
  }
  .s2 {
    transform: translate(145px, 70px);
  }

  /* States */
  .state-ask .ear-left {
    animation: ear-twitch 1.6s ease-in-out infinite;
  }
  @keyframes ear-twitch {
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
  .state-happy .pupil {
    display: none;
  }
  .state-happy .body-svg {
    animation: happy-jump 0.5s ease-in-out 2;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 650ms ease-out;
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
      transform: translateY(-10px);
    }
  }

  .state-sad .ear-left,
  .state-sad .ear-right {
    transform: rotate(0deg) translateY(4px);
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
    transform-origin: 138px 105px;
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
      breathe 2.5s ease-in-out infinite,
      hop-walk 400ms ease-in-out infinite;
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
