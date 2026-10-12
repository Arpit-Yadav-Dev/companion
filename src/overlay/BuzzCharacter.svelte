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
    idleGestures: ["spin", "dip", "wiggle"],
    eyeHeightRatio: 0.4,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 90 118 Q 100 132 110 118",
    shrug: "M 94 120 Q 100 118 106 120",
    sad: "M 94 122 Q 100 115 106 122",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 94 119 Q 100 126 106 119");
</script>

<div
  class="buzz state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <radialGradient id="buzz-body" cx="36%" cy="28%" r="76%">
        <stop offset="0%" stop-color="var(--gold-light)" />
        <stop offset="62%" stop-color="var(--gold)" />
        <stop offset="100%" stop-color="var(--gold-deep)" />
      </radialGradient>
    </defs>

    <!-- Hovering, so the shadow stays small and soft -->
    <ellipse class="ground-shadow" cx="100" cy="198" rx="24" ry="5" />

    <g class="hover-group">
      <!-- Wings behind the body, blurred by speed -->
      <g class="wings">
        <ellipse class="wing wing-left" cx="62" cy="80" rx="26" ry="16" />
        <ellipse class="wing wing-right" cx="138" cy="80" rx="26" ry="16" />
      </g>

      <g class="body-squash">
        <!-- One big round body: no neck, baby proportions -->
        <ellipse class="body" cx="100" cy="112" rx="52" ry="48" />
        <!-- Stripes follow the body's curve -->
        <path class="stripe" d="M 62 134 Q 100 146 138 134 Q 136 144 132 150 Q 100 160 68 150 Q 64 144 62 134 Z" />
        <path class="stripe" d="M 54 108 Q 100 120 146 108 L 146 120 Q 100 132 54 120 Z" />

        <!-- Tiny stinger: small enough to be harmless -->
        <path class="stinger" d="M 100 160 L 96 172 L 104 172 Z" />

        <ellipse class="cheek" cx="68" cy="118" rx="12" ry="8" />
        <ellipse class="cheek" cx="132" cy="118" rx="12" ry="8" />

        <!-- Oversized eyes, most of the face -->
        <g class="eye" class:closed={eyesClosed}>
          <ellipse class="eye-white" cx="80" cy="100" rx="16" ry="18" />
          <circle class="pupil" cx={80 + pupilOffset.x} cy={102 + pupilOffset.y} r="9" />
          <circle class="eye-shine big" cx={80 + pupilOffset.x - 4} cy={102 + pupilOffset.y - 6} r="4.5" />
          <circle class="eye-shine" cx={80 + pupilOffset.x + 4} cy={102 + pupilOffset.y + 3} r="2.2" />
          <path class="eyelid" d="M 64 100 Q 80 100 96 100" />
        </g>
        <g class="eye" class:closed={eyesClosed}>
          <ellipse class="eye-white" cx="120" cy="100" rx="16" ry="18" />
          <circle class="pupil" cx={120 + pupilOffset.x} cy={102 + pupilOffset.y} r="9" />
          <circle class="eye-shine big" cx={120 + pupilOffset.x - 4} cy={102 + pupilOffset.y - 6} r="4.5" />
          <circle class="eye-shine" cx={120 + pupilOffset.x + 4} cy={102 + pupilOffset.y + 3} r="2.2" />
          <path class="eyelid" d="M 104 100 Q 120 100 136 100" />
        </g>

        <path class="mouth" d={mouthPath} />
        <ellipse class="tear" cx="76" cy="116" rx="3" ry="5" />
      </g>

      <!-- Antennae with little bobbles -->
      <g class="antenna antenna-left">
        <path class="stem" d="M 86 66 Q 76 46 68 40" />
        <circle class="bobble" cx="66" cy="38" r="7" />
      </g>
      <g class="antenna antenna-right">
        <path class="stem" d="M 114 66 Q 124 46 132 40" />
        <circle class="bobble" cx="134" cy="38" r="7" />
      </g>

      {#if prop === "lookfar"}
        <circle class="prop-lookfar" cx="142" cy="92" r="11" />
      {:else if prop !== "stretch" && prop !== "posture"}
        <!-- A honey pot, carried along underneath -->
        <g class="prop-honey">
          <path d="M 142 146 Q 166 146 164 170 Q 142 176 140 170 Z" />
          <ellipse class="honey-lid" cx="152" cy="146" rx="13" ry="4.5" />
        </g>
      {/if}
    </g>

    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .buzz {
    --scale: 1;
    --gold-light: #ffe483;
    --gold: #ffc93c;
    --gold-deep: #e09c12;
    --stripe: #3b2f22;
    --wing: #d8f1ff;
    --cheek: #ff9ec7;

    position: relative;
    width: calc(150px * var(--scale));
    height: calc(161px * var(--scale));
    transform-origin: bottom center;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    transform-origin: 50% 100%;
  }

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.12);
    filter: blur(2px);
  }

  /* Never lands — a constant gentle hover is the resting state. */
  .hover-group {
    animation: hover 2.2s ease-in-out infinite;
    transform-origin: 100px 112px;
  }
  @keyframes hover {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-7px);
    }
  }

  .body {
    fill: url(#buzz-body);
    stroke: var(--gold-deep);
    stroke-width: 2;
  }
  .stripe {
    fill: var(--stripe);
    opacity: 0.88;
  }
  .stinger {
    fill: var(--stripe);
  }

  /* Wings beat too fast to see clearly, so they stay semi-transparent
     and blurred rather than drawn as solid shapes. */
  .wing {
    fill: var(--wing);
    opacity: 0.6;
    filter: blur(0.6px);
    animation: flutter 90ms ease-in-out infinite;
  }
  .wing-left {
    transform-origin: 84px 82px;
  }
  .wing-right {
    transform-origin: 116px 82px;
  }
  @keyframes flutter {
    0%,
    100% {
      transform: scaleY(1) rotate(0deg);
    }
    50% {
      transform: scaleY(0.6) rotate(-6deg);
    }
  }

  .antenna .stem {
    fill: none;
    stroke: var(--stripe);
    stroke-width: 3.5;
    stroke-linecap: round;
  }
  .antenna .bobble {
    fill: var(--stripe);
  }
  .antenna-left {
    transform-origin: 86px 66px;
    animation: antenna-sway 2.6s ease-in-out infinite;
  }
  .antenna-right {
    transform-origin: 114px 66px;
    animation: antenna-sway 2.6s ease-in-out infinite reverse;
  }
  @keyframes antenna-sway {
    0%,
    100% {
      transform: rotate(-5deg);
    }
    50% {
      transform: rotate(6deg);
    }
  }

  .eye-white {
    fill: white;
    stroke: var(--gold-deep);
    stroke-width: 1.6;
  }
  .pupil {
    fill: #2b2318;
    transition:
      cx 80ms linear,
      cy 80ms linear;
  }
  .eye-shine {
    fill: white;
    opacity: 0.95;
    transition:
      cx 80ms linear,
      cy 80ms linear;
  }
  .eye-shine.big {
    opacity: 1;
  }
  .eyelid {
    fill: none;
    stroke: var(--gold-deep);
    stroke-width: 3.5;
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

  .mouth {
    fill: none;
    stroke: #6b4a22;
    stroke-width: 3;
    stroke-linecap: round;
  }
  .cheek {
    fill: var(--cheek);
    opacity: 0.5;
  }
  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-honey path {
    fill: #f0a500;
    stroke: #c07d00;
    stroke-width: 1.5;
  }
  .honey-lid {
    fill: #fdf3d8;
    stroke: #c07d00;
    stroke-width: 1.5;
  }
  .prop-lookfar {
    fill: var(--gold);
    stroke: var(--gold-deep);
    stroke-width: 1.5;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #fff3b0;
  }
  .s1 {
    transform: translate(42px, 48px);
  }
  .s2 {
    transform: translate(158px, 72px);
  }

  /* --- Flies in on a bobbing path, tilting into the direction of
         travel, like something small fighting a breeze. --- */
  .state-walk_in .hover-group,
  .state-walk_out .hover-group {
    animation: fly calc(700ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes fly {
    0%,
    100% {
      transform: translateY(0) rotate(-7deg);
    }
    50% {
      transform: translateY(-16px) rotate(7deg);
    }
  }

  /* --- States --- */
  .state-ask .prop-honey,
  .state-ask .prop-lookfar {
    animation: prop-bob 1.6s ease-in-out infinite;
  }
  @keyframes prop-bob {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-3px) rotate(-5deg);
    }
  }

  /* Happy: eyes squeeze into arcs, body does an excited loop-the-loop */
  .state-happy .eye .eyelid {
    opacity: 1;
    transform: translateY(-4px) scaleY(1.4);
  }
  .state-happy .eye-white,
  .state-happy .pupil,
  .state-happy .eye-shine {
    display: none;
  }
  .state-happy .hover-group {
    animation: happy-loop 620ms ease-in-out infinite;
  }
  .state-happy .antenna-left,
  .state-happy .antenna-right {
    animation: antenna-excited 220ms ease-in-out infinite;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes happy-loop {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    35% {
      transform: translateY(-18px) rotate(-14deg);
    }
    70% {
      transform: translateY(-6px) rotate(14deg);
    }
  }
  @keyframes antenna-excited {
    0%,
    100% {
      transform: rotate(-14deg);
    }
    50% {
      transform: rotate(14deg);
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

  /* Antennae droop — the bee version of a shrug */
  .state-shrug .antenna-left {
    animation: none;
    transform: rotate(32deg);
  }
  .state-shrug .antenna-right {
    animation: none;
    transform: rotate(-32deg);
  }

  .state-sad .hover-group {
    animation: sad-sink 2.4s ease-in-out infinite;
  }
  .state-sad .antenna-left {
    animation: none;
    transform: rotate(44deg);
  }
  .state-sad .antenna-right {
    animation: none;
    transform: rotate(-44deg);
  }
  .state-sad .wing {
    animation-duration: 220ms;
    opacity: 0.4;
  }
  .state-sad .tear {
    opacity: 1;
    animation: tear-drop 1.4s ease-in infinite;
  }
  @keyframes sad-sink {
    0%,
    100% {
      transform: translateY(10px);
    }
    50% {
      transform: translateY(14px);
    }
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

  /* No arms, so it waggles its whole body to wave */
  .state-wave .hover-group {
    animation: body-wave 380ms ease-in-out 4;
  }
  @keyframes body-wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    25% {
      transform: rotate(-16deg);
    }
    75% {
      transform: rotate(16deg);
    }
  }

  /* --- Idle gestures --- */
  .gesture-spin.state-idle .hover-group {
    animation: gesture-spin 1300ms ease-in-out;
  }
  @keyframes gesture-spin {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(360deg);
    }
  }

  .gesture-dip.state-idle .hover-group {
    animation: gesture-dip 1200ms ease-in-out;
  }
  @keyframes gesture-dip {
    0%,
    100% {
      transform: translateY(0);
    }
    40% {
      transform: translateY(16px) scaleY(0.92);
    }
  }

  .gesture-wiggle.state-idle .body-squash {
    animation: gesture-wiggle 180ms ease-in-out 6;
    transform-origin: 100px 150px;
  }
  @keyframes gesture-wiggle {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(4deg);
    }
  }
</style>
