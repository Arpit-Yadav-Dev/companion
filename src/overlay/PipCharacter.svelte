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
    idleGestures: ["flap", "lookaround", "shuffle"],
    eyeHeightRatio: 0.26,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);
  let isWalking = $derived(behaviour.isWalking);
</script>

<div
  class="pip state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <radialGradient id="pip-back" cx="40%" cy="22%" r="80%">
        <stop offset="0%" stop-color="var(--suit-light)" />
        <stop offset="60%" stop-color="var(--suit)" />
        <stop offset="100%" stop-color="var(--suit-dark)" />
      </radialGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="202" rx="40" ry="7" />

    <!-- Orange feet, splayed wide — the waddle reads from these -->
    <g class="foot foot-left">
      <path d="M 86 196 Q 66 198 62 206 Q 78 210 90 204 Z" />
    </g>
    <g class="foot foot-right">
      <path d="M 114 196 Q 134 198 138 206 Q 122 210 110 204 Z" />
    </g>

    <g class="body-squash">
      <!-- One continuous egg: head and body are a single shape, so there
           is no neck — the silhouette cue that reads "penguin". -->
      <path
        class="back"
        d="M 100 34
           C 136 34 152 78 152 126
           C 152 174 130 198 100 198
           C 70 198 48 174 48 126
           C 48 78 64 34 100 34 Z"
      />
      <!-- Pale belly panel -->
      <path
        class="belly"
        d="M 100 66
           C 124 66 134 100 134 136
           C 134 172 118 190 100 190
           C 82 190 66 172 66 136
           C 66 100 76 66 100 66 Z"
      />

      <!-- Flippers hinge at the shoulder and swing with each waddle -->
      <g class="flipper flipper-left">
        <path d="M 52 104 Q 32 122 36 156 Q 50 158 58 132 Z" />
      </g>
      <g class="flipper flipper-right">
        <path d="M 148 104 Q 168 122 164 156 Q 150 158 142 132 Z" />
      </g>
    </g>

    <g class="head-group">
      <ellipse class="head-shine" cx="82" cy="58" rx="15" ry="11" />

      <ellipse class="blush" cx="70" cy="86" rx="9" ry="5" />
      <ellipse class="blush" cx="130" cy="86" rx="9" ry="5" />

      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-white" cx="84" cy="74" r="11" />
        <circle class="pupil" cx={84 + pupilOffset.x} cy={74 + pupilOffset.y} r="5.5" />
        <circle class="eye-shine" cx={84 + pupilOffset.x - 2} cy={74 + pupilOffset.y - 2.5} r="2" />
        <path class="eyelid" d="M 73 74 Q 84 74 95 74" />
      </g>
      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-white" cx="116" cy="74" r="11" />
        <circle class="pupil" cx={116 + pupilOffset.x} cy={74 + pupilOffset.y} r="5.5" />
        <circle class="eye-shine" cx={116 + pupilOffset.x - 2} cy={74 + pupilOffset.y - 2.5} r="2" />
        <path class="eyelid" d="M 105 74 Q 116 74 127 74" />
      </g>

      <!-- Short triangular beak -->
      <path class="beak beak-top" d="M 88 90 L 112 90 L 100 104 Z" />
      <path class="beak beak-bottom" d="M 92 101 L 108 101 L 100 108 Z" />

      <ellipse class="tear" cx="88" cy="86" rx="3" ry="5" />

      {#if prop === "lookfar"}
        <!-- Flipper raised to the brow, peering into the distance -->
        <path class="prop-lookfar" d="M 124 62 Q 142 56 148 66 Q 136 72 124 70 Z" />
      {:else if prop !== "stretch" && prop !== "posture"}
        <!-- An ice cube in a glass: the penguin's take on the drink prop -->
        <g class="prop-ice">
          <path d="M 150 128 L 170 128 L 166 156 L 154 156 Z" />
          <rect x="155" y="134" width="9" height="9" rx="2" />
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
  .pip {
    --scale: 1;
    --suit-light: #5a6475;
    --suit: #343c4c;
    --suit-dark: #1d2330;
    --belly: #fdfbf4;
    --beak: #f6a93b;
    --beak-dark: #d4822a;
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
    animation: breathe 3s ease-in-out infinite;
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

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.16);
    filter: blur(1.5px);
  }

  .back {
    fill: url(#pip-back);
    stroke: var(--suit-dark);
    stroke-width: 2;
  }
  .belly {
    fill: var(--belly);
  }
  .head-shine {
    fill: rgba(255, 255, 255, 0.22);
    filter: blur(3px);
  }

  .flipper path {
    fill: var(--suit);
    stroke: var(--suit-dark);
    stroke-width: 2;
  }
  .flipper-left {
    transform-origin: 54px 108px;
  }
  .flipper-right {
    transform-origin: 146px 108px;
  }

  .foot path {
    fill: var(--beak);
    stroke: var(--beak-dark);
    stroke-width: 2;
  }
  .foot-left {
    transform-origin: 88px 200px;
  }
  .foot-right {
    transform-origin: 112px 200px;
  }

  .beak {
    fill: var(--beak);
    stroke: var(--beak-dark);
    stroke-width: 1.5;
  }
  .beak-bottom {
    fill: var(--beak-dark);
  }

  .eye-white {
    fill: white;
    stroke: var(--suit-dark);
    stroke-width: 1.5;
  }
  .pupil {
    fill: #1b1f28;
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
  .eyelid {
    fill: none;
    stroke: var(--suit-dark);
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

  .blush {
    fill: var(--blush);
    opacity: 0.45;
  }
  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-ice path {
    fill: rgba(190, 228, 245, 0.8);
    stroke: #7fb4cc;
    stroke-width: 1.5;
  }
  .prop-ice rect {
    fill: #e8f6fd;
    stroke: #7fb4cc;
    stroke-width: 1.2;
  }
  .prop-lookfar {
    fill: var(--suit);
    stroke: var(--suit-dark);
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
    transform: translate(160px, 70px);
  }

  /* --- Waddle: the whole body rocks side to side, with the outer foot
         lifting on each tilt. No leg swing, because a penguin's legs are
         hidden inside the body — the rock IS the walk. --- */
  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 3s ease-in-out infinite,
      waddle calc(520ms / var(--anim-speed, 1)) ease-in-out infinite;
    transform-origin: 100px 200px;
  }
  @keyframes waddle {
    0%,
    100% {
      transform: rotate(-7deg) translateY(0);
    }
    25% {
      transform: rotate(-3deg) translateY(-3px);
    }
    50% {
      transform: rotate(7deg) translateY(0);
    }
    75% {
      transform: rotate(3deg) translateY(-3px);
    }
  }
  .state-walk_in .foot-left,
  .state-walk_out .foot-left {
    animation: foot-lift-a calc(520ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .foot-right,
  .state-walk_out .foot-right {
    animation: foot-lift-b calc(520ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes foot-lift-a {
    0%,
    45%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    70% {
      transform: translateY(-6px) rotate(-8deg);
    }
  }
  @keyframes foot-lift-b {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    20% {
      transform: translateY(-6px) rotate(8deg);
    }
    45% {
      transform: translateY(0) rotate(0deg);
    }
  }
  .state-walk_in .flipper-left,
  .state-walk_out .flipper-left {
    animation: flipper-a calc(520ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .flipper-right,
  .state-walk_out .flipper-right {
    animation: flipper-b calc(520ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes flipper-a {
    0%,
    100% {
      transform: rotate(10deg);
    }
    50% {
      transform: rotate(-14deg);
    }
  }
  @keyframes flipper-b {
    0%,
    100% {
      transform: rotate(-10deg);
    }
    50% {
      transform: rotate(14deg);
    }
  }

  /* --- States --- */
  .state-ask .prop-ice,
  .state-ask .prop-lookfar {
    animation: prop-bob 1.7s ease-in-out infinite;
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

  .state-happy .eye .eyelid {
    opacity: 1;
    transform: translateY(-3px) scaleY(1.3);
  }
  .state-happy .eye-white,
  .state-happy .pupil,
  .state-happy .eye-shine {
    display: none;
  }
  .state-happy .flipper-left {
    animation: flap-fast 220ms ease-in-out infinite;
  }
  .state-happy .flipper-right {
    animation: flap-fast-mirror 220ms ease-in-out infinite;
  }
  .state-happy .body-squash {
    animation: happy-bounce 400ms ease-in-out infinite;
    transform-origin: 100px 198px;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes flap-fast {
    0%,
    100% {
      transform: rotate(6deg);
    }
    50% {
      transform: rotate(-32deg);
    }
  }
  @keyframes flap-fast-mirror {
    0%,
    100% {
      transform: rotate(-6deg);
    }
    50% {
      transform: rotate(32deg);
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
      transform: scale(0.95, 1.07) translateY(-10px);
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

  .state-shrug .flipper-left {
    transform: rotate(-26deg);
  }
  .state-shrug .flipper-right {
    transform: rotate(26deg);
  }

  .state-sad .head-group {
    transform: translateY(5px);
  }
  .state-sad .flipper-left,
  .state-sad .flipper-right {
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

  .state-wave .flipper-right {
    animation: wave 520ms ease-in-out 3;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(46deg);
    }
  }

  /* --- Idle gestures --- */
  .gesture-flap.state-idle .flipper-left {
    animation: flap-fast 300ms ease-in-out 3;
  }
  .gesture-flap.state-idle .flipper-right {
    animation: flap-fast-mirror 300ms ease-in-out 3;
  }

  .head-group {
    transform-origin: 100px 120px;
  }
  .gesture-lookaround.state-idle .head-group {
    animation: gesture-look 1400ms ease-in-out;
  }
  @keyframes gesture-look {
    0%,
    100% {
      transform: rotate(0deg);
    }
    30% {
      transform: rotate(-9deg);
    }
    65% {
      transform: rotate(9deg);
    }
  }

  .gesture-shuffle.state-idle .body-svg {
    animation:
      breathe 3s ease-in-out infinite,
      gesture-shuffle 1200ms ease-in-out;
    transform-origin: 100px 200px;
  }
  @keyframes gesture-shuffle {
    0%,
    100% {
      transform: rotate(0deg);
    }
    25% {
      transform: rotate(-6deg);
    }
    75% {
      transform: rotate(6deg);
    }
  }
</style>
