<script lang="ts">
  import type { SceneState } from "./director";
  import { useCharacterBehaviour } from "./characterBehaviour.svelte";

  interface Props {
    sceneState: SceneState;
    prop?: string | null;
    scale?: number;
    mirrored?: boolean;
  }

  let { sceneState, prop = null, scale = 1, mirrored = false }: Props = $props();

  let rootEl: HTMLDivElement | undefined = $state();

  const behaviour = useCharacterBehaviour({
    sceneState: () => sceneState,
    mirrored: () => mirrored,
    rootEl: () => rootEl,
    idleGestures: ["scan", "beep", "recalibrate"],
    eyeHeightRatio: 0.3,
    maxPupilOffset: 5,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  // A robot "mouth" is a readout bar, not lips — the waveform changes
  // shape per mood instead of curving like a smile.
  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 84 124 L 90 116 L 96 128 L 102 114 L 108 126 L 116 120",
    shrug: "M 84 122 L 116 122",
    sad: "M 84 120 L 92 126 L 108 126 L 116 120",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 84 122 L 92 118 L 100 124 L 108 118 L 116 122");
</script>

<div
  class="bolt state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <linearGradient id="bolt-chassis" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="var(--metal-light)" />
        <stop offset="100%" stop-color="var(--metal)" />
      </linearGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="204" rx="44" ry="7" />

    <!-- Tracked base: no legs, so it rolls -->
    <g class="track-base">
      <rect class="track" x="54" y="176" width="92" height="26" rx="13" />
      <circle class="wheel wheel-1" cx="70" cy="189" r="8" />
      <circle class="wheel wheel-2" cx="100" cy="189" r="8" />
      <circle class="wheel wheel-3" cx="130" cy="189" r="8" />
    </g>

    <g class="chassis-group">
      <!-- Boxy torso with a panel and status light -->
      <rect class="chassis" x="60" y="108" width="80" height="70" rx="10" />
      <rect class="panel" x="72" y="122" width="56" height="34" rx="6" />
      <circle class="status-light" cx="100" cy="166" r="5" />

      <!-- Segmented arms, pinned at the shoulders -->
      <g class="arm arm-left">
        <rect class="arm-seg" x="38" y="116" width="16" height="34" rx="7" />
        <rect class="claw" x="36" y="148" width="20" height="13" rx="4" />
      </g>
      <g class="arm arm-right">
        <rect class="arm-seg" x="146" y="116" width="16" height="34" rx="7" />
        <rect class="claw" x="144" y="148" width="20" height="13" rx="4" />
      </g>
    </g>

    <g class="head-group">
      <!-- Antenna with a blinking bulb -->
      <line class="antenna-stem" x1="100" y1="58" x2="100" y2="34" />
      <circle class="antenna-bulb" cx="100" cy="30" r="7" />

      <rect class="head" x="60" y="58" width="80" height="54" rx="14" />
      <rect class="visor" x="68" y="66" width="64" height="30" rx="10" />

      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-glow" cx={84 + pupilOffset.x} cy={81 + pupilOffset.y} r="7" />
        <rect class="eyelid" x="70" y="78" width="28" height="6" rx="3" />
      </g>
      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-glow" cx={116 + pupilOffset.x} cy={81 + pupilOffset.y} r="7" />
        <rect class="eyelid" x="102" y="78" width="28" height="6" rx="3" />
      </g>

      <path class="mouth" d={mouthPath} />

      <!-- Side vents give the head some mass -->
      <path class="vent" d="M 64 100 L 74 100 M 64 106 L 74 106" />
      <path class="vent" d="M 126 100 L 136 100 M 126 106 L 136 106" />
    </g>

    {#if prop === "lookfar"}
      <!-- A telescoping scanner extends from the visor -->
      <g class="prop-scanner">
        <rect x="140" y="74" width="26" height="9" rx="4" />
        <circle cx="170" cy="78" r="6" />
      </g>
    {:else if prop !== "stretch" && prop !== "posture"}
      <!-- Oil can, clamped in the right claw -->
      <g class="prop-oil">
        <rect x="146" y="160" width="20" height="22" rx="4" />
        <path d="M 166 166 L 178 158" />
      </g>
    {/if}

    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .bolt {
    --scale: 1;
    --metal-light: #c6d2dc;
    --metal: #93a4b3;
    --metal-dark: #5a6b7a;
    --panel: #2f3b47;
    --glow: #4fe3c1;
    --accent: #ffb13b;

    position: relative;
    width: calc(150px * var(--scale));
    height: calc(161px * var(--scale));
    transform-origin: bottom center;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    /* A machine hums rather than breathes — a much subtler idle than
       the animals' squash-and-stretch. */
    animation: hum 2.2s ease-in-out infinite;
    transform-origin: 50% 100%;
  }

  @keyframes hum {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-1.5px);
    }
  }

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.16);
    filter: blur(1.5px);
  }

  .chassis,
  .head {
    fill: url(#bolt-chassis);
    stroke: var(--metal-dark);
    stroke-width: 2;
  }
  .panel,
  .visor {
    fill: var(--panel);
  }
  .track {
    fill: var(--panel);
    stroke: var(--metal-dark);
    stroke-width: 2;
  }
  .wheel {
    fill: var(--metal);
    stroke: var(--metal-dark);
    stroke-width: 2;
  }

  .arm-seg,
  .claw {
    fill: var(--metal);
    stroke: var(--metal-dark);
    stroke-width: 2;
  }
  .arm-left {
    transform-origin: 46px 118px;
  }
  .arm-right {
    transform-origin: 154px 118px;
  }

  .antenna-stem {
    stroke: var(--metal-dark);
    stroke-width: 3;
    stroke-linecap: round;
  }
  .antenna-bulb {
    fill: var(--accent);
    animation: blink-bulb 2.4s ease-in-out infinite;
  }
  @keyframes blink-bulb {
    0%,
    80%,
    100% {
      opacity: 0.45;
    }
    90% {
      opacity: 1;
    }
  }

  .eye-glow {
    fill: var(--glow);
    filter: drop-shadow(0 0 3px var(--glow));
    transition:
      cx 80ms linear,
      cy 80ms linear;
  }
  .eyelid {
    fill: var(--panel);
    opacity: 0;
  }
  .eye.closed .eye-glow {
    opacity: 0;
  }
  .eye.closed .eyelid {
    opacity: 1;
  }

  .mouth {
    fill: none;
    stroke: var(--glow);
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .vent {
    stroke: var(--metal-dark);
    stroke-width: 2;
    stroke-linecap: round;
  }
  .status-light {
    fill: var(--glow);
    animation: blink-bulb 1.8s ease-in-out infinite;
  }

  .prop-oil rect {
    fill: #e8503a;
    stroke: #b83c2a;
    stroke-width: 1.5;
  }
  .prop-oil path {
    stroke: #b83c2a;
    stroke-width: 3;
    stroke-linecap: round;
  }
  .prop-scanner rect,
  .prop-scanner circle {
    fill: var(--metal);
    stroke: var(--metal-dark);
    stroke-width: 1.5;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: var(--glow);
  }
  .s1 {
    transform: translate(44px, 44px);
  }
  .s2 {
    transform: translate(158px, 70px);
  }

  /* --- Rolls in on its tracks: wheels spin, chassis stays level, and
         only the antenna bobs. Deliberately mechanical — no bounce. --- */
  .state-walk_in .wheel,
  .state-walk_out .wheel {
    animation: spin calc(500ms / var(--anim-speed, 1)) linear infinite;
    transform-box: fill-box;
    transform-origin: center;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .state-walk_in .chassis-group,
  .state-walk_out .chassis-group,
  .state-walk_in .head-group,
  .state-walk_out .head-group {
    animation: chassis-jitter calc(260ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes chassis-jitter {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-1.5px);
    }
  }
  .state-walk_in .antenna-bulb,
  .state-walk_out .antenna-bulb {
    animation: antenna-bob calc(500ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes antenna-bob {
    0%,
    100% {
      transform: translateX(-3px);
    }
    50% {
      transform: translateX(3px);
    }
  }

  /* --- States --- */
  .state-ask .prop-oil,
  .state-ask .prop-scanner {
    animation: prop-bob 1.7s ease-in-out infinite;
  }
  @keyframes prop-bob {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-3px) rotate(-4deg);
    }
  }

  /* Everything lights up and the arms snap to a victory raise */
  .state-happy .antenna-bulb,
  .state-happy .status-light {
    animation: flash 180ms steps(2) infinite;
  }
  .state-happy .eye-glow {
    animation: eye-pulse 300ms ease-in-out infinite;
  }
  .state-happy .arm-left {
    animation: arm-up-left 420ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
  .state-happy .arm-right {
    animation: arm-up-right 420ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes flash {
    0% {
      opacity: 1;
    }
    100% {
      opacity: 0.3;
    }
  }
  @keyframes eye-pulse {
    0%,
    100% {
      r: 7;
    }
    50% {
      r: 9;
    }
  }
  @keyframes arm-up-left {
    to {
      transform: rotate(-54deg);
    }
  }
  @keyframes arm-up-right {
    to {
      transform: rotate(54deg);
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

  /* A glitchy head-shake instead of a shrug */
  .state-shrug .head-group {
    animation: glitch-shake 420ms ease-in-out 2;
  }
  .state-shrug .mouth {
    opacity: 0.6;
  }
  @keyframes glitch-shake {
    0%,
    100% {
      transform: translateX(0);
    }
    25% {
      transform: translateX(-4px) skewX(3deg);
    }
    75% {
      transform: translateX(4px) skewX(-3deg);
    }
  }

  /* Powers down: lights dim and the head droops */
  .state-sad .head-group {
    transform: translateY(6px) rotate(-3deg);
  }
  .state-sad .eye-glow,
  .state-sad .antenna-bulb,
  .state-sad .status-light {
    opacity: 0.3;
    animation: none;
  }
  .state-sad .arm-left,
  .state-sad .arm-right {
    transform: translateY(5px);
  }

  .state-wave .arm-right {
    animation: wave 500ms ease-in-out 3;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(40deg);
    }
    50% {
      transform: rotate(72deg);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 112px;
  }
  .gesture-scan.state-idle .head-group {
    animation: scan 1500ms ease-in-out;
  }
  @keyframes scan {
    0%,
    100% {
      transform: rotate(0deg);
    }
    30% {
      transform: rotate(-11deg);
    }
    70% {
      transform: rotate(11deg);
    }
  }

  .gesture-beep.state-idle .mouth {
    animation: beep 220ms steps(2) 5;
  }
  .gesture-beep.state-idle .antenna-bulb {
    animation: flash 220ms steps(2) 5;
  }
  @keyframes beep {
    0% {
      opacity: 1;
    }
    100% {
      opacity: 0.25;
    }
  }

  .gesture-recalibrate.state-idle .arm-left {
    animation: recalib 700ms ease-in-out 2;
  }
  @keyframes recalib {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-26deg);
    }
  }
</style>
