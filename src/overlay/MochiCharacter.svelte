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
    idleGestures: ["groom", "earflick", "stretch-back"],
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);
  let isWalking = $derived(behaviour.isWalking);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 88 150 Q 100 160 112 150",
    shrug: "M 90 152 Q 100 151 110 152",
    sad: "M 90 154 Q 100 148 110 154",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 92 150 Q 100 155 108 150");
</script>

<div
  class="mochi state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  {#if isWalking}
    <div class="dust-puffs">
      <span class="puff p1"></span>
      <span class="puff p2"></span>
      <span class="puff p3"></span>
    </div>
  {/if}

  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <radialGradient id="mochi-body" cx="38%" cy="28%" r="78%">
        <stop offset="0%" stop-color="var(--fur-light)" />
        <stop offset="62%" stop-color="var(--fur)" />
        <stop offset="100%" stop-color="var(--fur-shadow)" />
      </radialGradient>
      <radialGradient id="mochi-head" cx="36%" cy="26%" r="80%">
        <stop offset="0%" stop-color="var(--fur-light)" />
        <stop offset="60%" stop-color="var(--fur)" />
        <stop offset="100%" stop-color="var(--fur-shadow)" />
      </radialGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="204" rx="46" ry="8" />

    <!-- Tail: a long curl that sways, the most cat-like silhouette cue -->
    <g class="tail-root">
      <path class="tail" d="M 148 170 Q 182 164 178 128 Q 176 108 160 110" />
    </g>

    <!-- Hind legs -->
    <g class="leg leg-back-left" style="transform-origin: 80px 168px">
      <rect class="leg-seg" x="72" y="168" width="16" height="30" rx="8" />
      <ellipse class="paw" cx="80" cy="198" rx="12" ry="7" />
    </g>
    <g class="leg leg-back-right" style="transform-origin: 120px 168px">
      <rect class="leg-seg" x="112" y="168" width="16" height="30" rx="8" />
      <ellipse class="paw" cx="120" cy="198" rx="12" ry="7" />
    </g>

    <g class="body-squash">
      <ellipse class="body" cx="100" cy="155" rx="44" ry="38" />
      <ellipse class="belly" cx="100" cy="163" rx="24" ry="22" />
    </g>

    <!-- Front legs -->
    <g class="leg leg-front-left" style="transform-origin: 78px 160px">
      <rect class="leg-seg" x="71" y="160" width="14" height="34" rx="7" />
      <ellipse class="paw" cx="78" cy="194" rx="11" ry="6.5" />
    </g>
    <g class="leg leg-front-right" style="transform-origin: 122px 160px">
      <rect class="leg-seg" x="115" y="160" width="14" height="34" rx="7" />
      <ellipse class="paw" cx="122" cy="194" rx="11" ry="6.5" />
    </g>

    <g class="head-group">
      <!-- Pointed ears with inner pink -->
      <path class="ear" d="M 66 92 L 56 50 L 92 74 Z" />
      <path class="ear" d="M 134 92 L 144 50 L 108 74 Z" />
      <path class="ear-inner" d="M 70 88 L 63 60 L 86 76 Z" />
      <path class="ear-inner" d="M 130 88 L 137 60 L 114 76 Z" />

      <circle class="head" cx="100" cy="106" r="42" />
      <ellipse class="head-shine" cx="82" cy="88" rx="14" ry="10" />

      <ellipse class="muzzle" cx="100" cy="132" rx="22" ry="14" />

      <ellipse class="blush" cx="70" cy="122" rx="8" ry="5" />
      <ellipse class="blush" cx="130" cy="122" rx="8" ry="5" />

      <g class="eye" class:closed={eyesClosed}>
        <ellipse class="eye-white" cx="82" cy="104" rx="10" ry="11" />
        <ellipse
          class="pupil"
          cx={82 + pupilOffset.x}
          cy={104 + pupilOffset.y}
          rx="4"
          ry="8.5"
        />
        <circle class="eye-shine" cx={82 + pupilOffset.x - 1.6} cy={104 + pupilOffset.y - 3} r="1.7" />
        <path class="eyelid" d="M 72 104 Q 82 104 92 104" />
      </g>
      <g class="eye" class:closed={eyesClosed}>
        <ellipse class="eye-white" cx="118" cy="104" rx="10" ry="11" />
        <ellipse
          class="pupil"
          cx={118 + pupilOffset.x}
          cy={104 + pupilOffset.y}
          rx="4"
          ry="8.5"
        />
        <circle class="eye-shine" cx={118 + pupilOffset.x - 1.6} cy={104 + pupilOffset.y - 3} r="1.7" />
        <path class="eyelid" d="M 108 104 Q 118 104 128 104" />
      </g>

      <!-- Nose + mouth -->
      <path class="nose" d="M 94 124 L 106 124 L 100 131 Z" />
      <path class="mouth" d={mouthPath} />

      <!-- Whiskers -->
      <g class="whiskers">
        <path d="M 74 130 L 48 126" />
        <path d="M 74 136 L 50 138" />
        <path d="M 126 130 L 152 126" />
        <path d="M 126 136 L 150 138" />
      </g>

      <ellipse class="tear" cx="88" cy="114" rx="3" ry="5" />

      {#if prop === "lookfar"}
        <ellipse class="prop-lookfar" cx="126" cy="94" rx="10" ry="7" />
      {:else if prop !== "stretch" && prop !== "posture"}
        <!-- Milk saucer: the cat's take on the water-bottle prop -->
        <g class="prop-saucer">
          <ellipse cx="146" cy="140" rx="16" ry="6" />
          <path d="M 132 138 Q 146 148 160 138" />
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
  .mochi {
    --scale: 1;
    /* Warm ginger tabby — distinct from Scout's cool grey husky so the
       two read as different animals at thumbnail size. */
    --fur-light: #f6c98f;
    --fur: #e8a85c;
    --fur-shadow: #c07b35;
    --fur-dark: #8a5421;
    --muzzle: #fdf1e0;
    --blush: #ff9ec7;

    position: relative;
    width: calc(150px * var(--scale));
    height: calc(161px * var(--scale));
    transform-origin: bottom center;
  }

  .body-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
    animation: breathe 2.8s ease-in-out infinite;
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

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.16);
    filter: blur(1.5px);
  }

  .body {
    fill: url(#mochi-body);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .belly {
    fill: var(--muzzle);
    opacity: 0.85;
  }
  .head {
    fill: url(#mochi-head);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .head-shine {
    fill: rgba(255, 255, 255, 0.38);
    filter: blur(2px);
  }
  .muzzle {
    fill: var(--muzzle);
  }

  .leg-seg,
  .paw {
    fill: var(--fur);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }

  .ear {
    fill: var(--fur-shadow);
  }
  .ear-inner {
    fill: #f5a0b4;
  }

  .tail {
    fill: none;
    stroke: var(--fur);
    stroke-width: 13;
    stroke-linecap: round;
  }
  .tail-root {
    transform-origin: 148px 170px;
    animation: tail-idle 3.2s ease-in-out infinite;
  }
  @keyframes tail-idle {
    0%,
    100% {
      transform: rotate(-4deg);
    }
    50% {
      transform: rotate(6deg);
    }
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

  .nose {
    fill: #e2748f;
  }
  .mouth {
    fill: none;
    stroke: #8a5421;
    stroke-width: 3;
    stroke-linecap: round;
  }
  .whiskers {
    fill: none;
    stroke: rgba(80, 55, 30, 0.45);
    stroke-width: 1.6;
    stroke-linecap: round;
  }
  .blush {
    fill: var(--blush);
    opacity: 0.5;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-saucer ellipse {
    fill: #eef3f7;
    stroke: var(--fur-dark);
    stroke-width: 1.5;
  }
  .prop-saucer path {
    fill: none;
    stroke: var(--fur-dark);
    stroke-width: 1.5;
  }
  .prop-lookfar {
    fill: var(--fur);
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
    transform: translate(44px, 62px);
  }
  .s2 {
    transform: translate(158px, 84px);
  }

  /* --- Walk cycle: diagonal pairs, same gait model as Scout --- */
  .leg {
    transition: transform 180ms ease;
  }
  .state-walk_in .leg-front-left,
  .state-walk_out .leg-front-left,
  .state-walk_in .leg-back-right,
  .state-walk_out .leg-back-right {
    animation: swing-a calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .leg-front-right,
  .state-walk_out .leg-front-right,
  .state-walk_in .leg-back-left,
  .state-walk_out .leg-back-left {
    animation: swing-b calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes swing-a {
    0%,
    100% {
      transform: rotate(-20deg);
    }
    50% {
      transform: rotate(20deg);
    }
  }
  @keyframes swing-b {
    0%,
    100% {
      transform: rotate(20deg);
    }
    50% {
      transform: rotate(-20deg);
    }
  }

  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.8s ease-in-out infinite,
      walk-bob calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes walk-bob {
    0%,
    50%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    25% {
      transform: translateY(-4px) rotate(-1.5deg);
    }
    75% {
      transform: translateY(-4px) rotate(1.5deg);
    }
  }

  .state-walk_in .tail-root,
  .state-walk_out .tail-root {
    animation: tail-walk calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes tail-walk {
    0%,
    100% {
      transform: rotate(-10deg);
    }
    50% {
      transform: rotate(10deg);
    }
  }

  /* --- States --- */
  .state-ask .prop-saucer,
  .state-ask .prop-lookfar {
    animation: prop-bob 1.6s ease-in-out infinite;
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

  .state-happy .eye .eyelid {
    opacity: 1;
    transform: translateY(-3px) scaleY(1.3);
  }
  .state-happy .eye-white,
  .state-happy .pupil,
  .state-happy .eye-shine {
    display: none;
  }
  .state-happy .tail-root {
    animation: tail-fast 240ms ease-in-out infinite;
  }
  .state-happy .body-squash {
    animation: happy-bounce 380ms ease-in-out infinite;
    transform-origin: 100px 196px;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes tail-fast {
    0%,
    100% {
      transform: rotate(-16deg);
    }
    50% {
      transform: rotate(16deg);
    }
  }
  @keyframes happy-bounce {
    0%,
    100% {
      transform: scale(1, 1) translateY(0);
    }
    45% {
      transform: scale(1.05, 0.93) translateY(2px);
    }
    70% {
      transform: scale(0.96, 1.06) translateY(-8px);
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

  .state-shrug .leg-front-left,
  .state-shrug .leg-front-right {
    animation: shrug-legs 600ms ease-out both;
  }
  @keyframes shrug-legs {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(-7px);
    }
  }

  .state-sad .ear {
    transform: translateY(6px) scale(0.94);
    transform-origin: 100px 90px;
  }
  .state-sad .tail-root {
    transform: rotate(-22deg) translateY(10px);
  }
  .state-sad .tear {
    opacity: 1;
    animation: tear-drop 1.4s ease-in infinite;
  }
  .state-sad .head-group {
    transform: translateY(4px) rotate(-3deg);
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

  .state-wave .leg-front-right {
    animation: wave 500ms ease-in-out 3;
    transform-origin: 122px 160px;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-48deg);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 150px;
  }
  .gesture-groom.state-idle .head-group {
    animation: gesture-groom 1400ms ease-in-out;
  }
  @keyframes gesture-groom {
    0%,
    100% {
      transform: rotate(0deg) translateY(0);
    }
    35%,
    65% {
      transform: rotate(-14deg) translateY(8px);
    }
  }
  .gesture-groom.state-idle .leg-front-left {
    animation: gesture-groom-paw 1400ms ease-in-out;
    transform-origin: 78px 160px;
  }
  @keyframes gesture-groom-paw {
    0%,
    100% {
      transform: rotate(0deg);
    }
    35%,
    65% {
      transform: rotate(-58deg) translate(4px, -10px);
    }
  }

  .gesture-earflick.state-idle .ear {
    animation: gesture-earflick 700ms ease-in-out 2;
    transform-origin: 100px 90px;
  }
  @keyframes gesture-earflick {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-9deg);
    }
  }

  .gesture-stretch-back.state-idle .body-squash {
    animation: gesture-stretch 1400ms ease-in-out;
    transform-origin: 100px 196px;
  }
  @keyframes gesture-stretch {
    0%,
    100% {
      transform: scale(1, 1);
    }
    40%,
    60% {
      transform: scale(1.1, 0.9) translateY(4px);
    }
  }

  .dust-puffs {
    position: absolute;
    bottom: 3px;
    left: 50%;
    translate: -50% 0;
    width: 72px;
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
    left: 31px;
    animation-delay: 135ms;
  }
  .p3 {
    left: 52px;
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
