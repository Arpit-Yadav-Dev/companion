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
    idleGestures: ["snooze", "puff", "stretch"],
    eyeHeightRatio: 0.44,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 90 124 Q 100 138 110 124",
    shrug: "M 94 126 Q 100 124 106 126",
    sad: "M 94 128 Q 100 121 106 128",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 95 125 Q 100 131 105 125");
</script>

<div
  class="nimbus state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <radialGradient id="nimbus-fill" cx="36%" cy="26%" r="80%">
        <stop offset="0%" stop-color="#ffffff" />
        <stop offset="70%" stop-color="var(--cloud)" />
        <stop offset="100%" stop-color="var(--cloud-shade)" />
      </radialGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="196" rx="30" ry="6" />

    <g class="drift-group">
      <!-- Rain droplets, only shown when sad -->
      <g class="rain">
        <path class="drop d1" d="M 76 150 q -4 7 0 10 q 4 -3 0 -10 Z" />
        <path class="drop d2" d="M 100 154 q -4 7 0 10 q 4 -3 0 -10 Z" />
        <path class="drop d3" d="M 124 150 q -4 7 0 10 q 4 -3 0 -10 Z" />
      </g>

      <g class="body-squash">
        <!-- Overlapping circles make a cloud read instantly -->
        <g class="puffs">
          <circle class="puff" cx="68" cy="112" r="28" />
          <circle class="puff" cx="132" cy="112" r="28" />
          <circle class="puff" cx="84" cy="88" r="30" />
          <circle class="puff" cx="116" cy="88" r="30" />
          <circle class="puff" cx="100" cy="76" r="26" />
          <rect class="puff" x="66" y="100" width="68" height="34" rx="17" />
        </g>

        <ellipse class="cheek" cx="72" cy="118" rx="11" ry="7" />
        <ellipse class="cheek" cx="128" cy="118" rx="11" ry="7" />

        <!-- Sleepy half-lidded eyes: curved lines, not circles -->
        <g class="eye" class:closed={eyesClosed}>
          <ellipse class="eye-white" cx="82" cy="106" rx="13" ry="14" />
          <circle class="pupil" cx={82 + pupilOffset.x} cy={108 + pupilOffset.y} r="7.5" />
          <circle class="eye-shine big" cx={82 + pupilOffset.x - 3} cy={108 + pupilOffset.y - 5} r="3.6" />
          <!-- Permanent droopy lid across the top -->
          <path class="sleepy-lid" d="M 69 100 Q 82 93 95 100 L 95 94 Q 82 88 69 94 Z" />
          <path class="eyelid" d="M 69 106 Q 82 106 95 106" />
        </g>
        <g class="eye" class:closed={eyesClosed}>
          <ellipse class="eye-white" cx="118" cy="106" rx="13" ry="14" />
          <circle class="pupil" cx={118 + pupilOffset.x} cy={108 + pupilOffset.y} r="7.5" />
          <circle class="eye-shine big" cx={118 + pupilOffset.x - 3} cy={108 + pupilOffset.y - 5} r="3.6" />
          <path class="sleepy-lid" d="M 105 100 Q 118 93 131 100 L 131 94 Q 118 88 105 94 Z" />
          <path class="eyelid" d="M 105 106 Q 118 106 131 106" />
        </g>

        <path class="mouth" d={mouthPath} />
      </g>

      <!-- Floating Zs while idle -->
      <g class="zees">
        <text class="zee z1" x="146" y="62">z</text>
        <text class="zee z2" x="156" y="46">z</text>
        <text class="zee z3" x="166" y="32">z</text>
      </g>

      {#if prop === "lookfar"}
        <ellipse class="prop-lookfar" cx="142" cy="94" rx="12" ry="8" />
      {:else if prop !== "stretch" && prop !== "posture"}
        <!-- A little raincloud spout pouring into a glass -->
        <g class="prop-glass">
          <path d="M 142 146 L 164 146 L 160 176 L 146 176 Z" />
          <path class="water" d="M 145 158 L 161 158 L 159 174 L 147 174 Z" />
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
  .nimbus {
    --scale: 1;
    --cloud: #eef4fb;
    --cloud-shade: #c9d8ea;
    --cloud-edge: #a8bdd4;
    --cheek: #ffb3c8;
    --water: #6ec6f0;

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
    fill: rgba(0, 0, 0, 0.1);
    filter: blur(2.5px);
  }

  /* A cloud drifts; it never has a walk cycle. */
  .drift-group {
    animation: drift 4.2s ease-in-out infinite;
    transform-origin: 100px 104px;
  }
  @keyframes drift {
    0%,
    100% {
      transform: translateY(0) translateX(0);
    }
    50% {
      transform: translateY(-9px) translateX(3px);
    }
  }

  .puff {
    fill: url(#nimbus-fill);
    stroke: var(--cloud-edge);
    stroke-width: 2;
  }
  /* The outline would otherwise show through where circles overlap. */
  .puffs {
    filter: drop-shadow(0 2px 2px rgba(120, 150, 180, 0.18));
  }
  .puffs .puff + .puff {
    stroke: none;
  }

  .eye-white {
    fill: white;
    stroke: var(--cloud-edge);
    stroke-width: 1.5;
  }
  .pupil {
    fill: #46566b;
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
  /* Half-closed lids are what make it read as sleepy rather than blank. */
  .sleepy-lid {
    fill: var(--cloud);
    stroke: var(--cloud-edge);
    stroke-width: 1.5;
  }
  .eyelid {
    fill: none;
    stroke: var(--cloud-edge);
    stroke-width: 3;
    stroke-linecap: round;
    opacity: 0;
  }
  .eye.closed .eye-white,
  .eye.closed .pupil,
  .eye.closed .eye-shine,
  .eye.closed .sleepy-lid {
    opacity: 0;
  }
  .eye.closed .eyelid {
    opacity: 1;
  }

  .mouth {
    fill: none;
    stroke: #7a8da3;
    stroke-width: 2.8;
    stroke-linecap: round;
  }
  .cheek {
    fill: var(--cheek);
    opacity: 0.5;
  }

  .zee {
    fill: #9db4cc;
    font-family: var(--font-display, sans-serif);
    font-weight: 700;
    opacity: 0;
  }
  .z1 {
    font-size: 16px;
  }
  .z2 {
    font-size: 13px;
  }
  .z3 {
    font-size: 10px;
  }

  .rain {
    opacity: 0;
  }
  .drop {
    fill: var(--water);
  }

  .prop-glass path {
    fill: rgba(220, 240, 250, 0.55);
    stroke: #9ec4d8;
    stroke-width: 1.5;
  }
  .prop-glass .water {
    fill: var(--water);
    opacity: 0.8;
  }
  .prop-lookfar {
    fill: var(--cloud);
    stroke: var(--cloud-edge);
    stroke-width: 1.5;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(44px, 44px);
  }
  .s2 {
    transform: translate(158px, 68px);
  }

  /* --- Floats in slowly, stretching and squashing like something soft
         with no fixed shape. --- */
  .state-walk_in .drift-group,
  .state-walk_out .drift-group {
    animation: float-in calc(1200ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes float-in {
    0%,
    100% {
      transform: translateY(0) scale(1, 1);
    }
    35% {
      transform: translateY(-12px) scale(1.05, 0.95);
    }
    70% {
      transform: translateY(-4px) scale(0.97, 1.03);
    }
  }

  /* --- States --- */
  .state-ask .prop-glass,
  .state-ask .prop-lookfar {
    animation: prop-bob 1.8s ease-in-out infinite;
  }
  @keyframes prop-bob {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-3px) rotate(-3deg);
    }
  }

  /* Wakes up properly: lids lift and it bounces for once */
  .state-happy .sleepy-lid {
    opacity: 0;
  }
  .state-happy .eye .eyelid {
    opacity: 1;
    transform: translateY(-4px) scaleY(1.4);
  }
  .state-happy .eye-white,
  .state-happy .pupil,
  .state-happy .eye-shine {
    display: none;
  }
  .state-happy .drift-group {
    animation: happy-bounce 460ms ease-in-out infinite;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes happy-bounce {
    0%,
    100% {
      transform: translateY(0) scale(1, 1);
    }
    45% {
      transform: translateY(-16px) scale(0.96, 1.06);
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

  .state-shrug .body-squash {
    animation: shrug-squish 600ms ease-out both;
    transform-origin: 100px 130px;
  }
  @keyframes shrug-squish {
    to {
      transform: scale(1.06, 0.92);
    }
  }

  /* A sad cloud rains. Nothing else needed. */
  .state-sad .rain {
    opacity: 1;
  }
  .state-sad .d1 {
    animation: rain-fall 900ms linear infinite;
  }
  .state-sad .d2 {
    animation: rain-fall 900ms linear infinite 300ms;
  }
  .state-sad .d3 {
    animation: rain-fall 900ms linear infinite 600ms;
  }
  .state-sad .drift-group {
    animation: sad-sink 3s ease-in-out infinite;
  }
  .state-sad .puff {
    filter: saturate(0.7) brightness(0.94);
  }
  @keyframes rain-fall {
    0% {
      opacity: 0;
      transform: translateY(0);
    }
    20% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: translateY(32px);
    }
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

  .state-wave .drift-group {
    animation: body-wave 440ms ease-in-out 4;
  }
  @keyframes body-wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    25% {
      transform: rotate(-12deg);
    }
    75% {
      transform: rotate(12deg);
    }
  }

  /* --- Idle gestures --- */
  /* Drifts off mid-conversation, with Zs rising */
  .gesture-snooze.state-idle .zee {
    animation: zee-rise 2.4s ease-out;
  }
  .gesture-snooze.state-idle .z2 {
    animation-delay: 300ms;
  }
  .gesture-snooze.state-idle .z3 {
    animation-delay: 600ms;
  }
  .gesture-snooze.state-idle .sleepy-lid {
    animation: lid-droop 2.4s ease-in-out;
  }
  @keyframes zee-rise {
    0% {
      opacity: 0;
      transform: translate(0, 6px) scale(0.6);
    }
    35% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: translate(10px, -22px) scale(1.1);
    }
  }
  @keyframes lid-droop {
    0%,
    100% {
      transform: translateY(0);
    }
    40%,
    60% {
      transform: translateY(9px);
    }
  }

  .gesture-puff.state-idle .body-squash {
    animation: puff-up 1200ms ease-in-out;
    transform-origin: 100px 110px;
  }
  @keyframes puff-up {
    0%,
    100% {
      transform: scale(1);
    }
    45% {
      transform: scale(1.07);
    }
  }

  .gesture-stretch.state-idle .body-squash {
    animation: wide-stretch 1300ms ease-in-out;
    transform-origin: 100px 130px;
  }
  @keyframes wide-stretch {
    0%,
    100% {
      transform: scale(1, 1);
    }
    45% {
      transform: scale(1.12, 0.9);
    }
  }
</style>
