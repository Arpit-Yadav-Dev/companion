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
    idleGestures: ["hairflick", "rock", "think"],
    eyeHeightRatio: 0.26,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 90 76 Q 100 88 110 76",
    shrug: "M 93 78 Q 100 76 107 78",
    sad: "M 93 80 Q 100 74 107 80",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 93 77 Q 100 83 107 77");
</script>

<div
  class="luna state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <linearGradient id="luna-dress" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="var(--dress-light)" />
        <stop offset="100%" stop-color="var(--dress)" />
      </linearGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="202" rx="36" ry="7" />

    <!-- Ponytail sits behind everything so it reads as "behind the head" -->
    <g class="ponytail">
      <path d="M 136 54 Q 164 62 160 96 Q 156 120 142 118 Q 150 92 136 70 Z" />
    </g>

    <g class="legs">
      <g class="leg leg-left">
        <rect class="shin" x="86" y="160" width="12" height="32" rx="6" />
        <ellipse class="shoe" cx="92" cy="195" rx="12" ry="6.5" />
      </g>
      <g class="leg leg-right">
        <rect class="shin" x="102" y="160" width="12" height="32" rx="6" />
        <ellipse class="shoe" cx="108" cy="195" rx="12" ry="6.5" />
      </g>
    </g>

    <g class="body-squash">
      <!-- A-line dress: narrow at the shoulders, flaring to the hem -->
      <path
        class="dress"
        d="M 100 96
           C 112 96 118 104 122 128
           L 132 166
           Q 100 176 68 166
           L 78 128
           C 82 104 88 96 100 96 Z"
      />
      <path class="collar" d="M 88 100 Q 100 108 112 100 L 110 96 Q 100 102 90 96 Z" />

      <g class="arm arm-left">
        <rect class="limb" x="62" y="106" width="12" height="34" rx="6" />
        <circle class="hand" cx="68" cy="144" r="7.5" />
      </g>
      <g class="arm arm-right">
        <rect class="limb" x="126" y="106" width="12" height="34" rx="6" />
        <circle class="hand" cx="132" cy="144" r="7.5" />

        {#if prop !== "lookfar"}
          <!-- Water bottle held in the right hand -->
          <g class="prop-bottle">
            <rect x="124" y="146" width="16" height="30" rx="4" />
            <rect x="127" y="139" width="10" height="8" rx="2.5" />
            <path d="M 126 160 L 138 160" />
          </g>
        {/if}
      </g>
    </g>

    <g class="head-group">
      <!-- Hair cap over the skull, with a fringe across the brow -->
      <path class="hair-back" d="M 66 62 Q 66 28 100 28 Q 134 28 134 62 L 134 76 Q 100 66 66 76 Z" />
      <ellipse class="head" cx="100" cy="68" rx="31" ry="33" />
      <path class="fringe" d="M 69 60 Q 76 40 100 40 Q 124 40 131 60 Q 116 50 100 54 Q 84 50 69 60 Z" />
      <ellipse class="head-shine" cx="88" cy="54" rx="9" ry="7" />

      <ellipse class="blush" cx="77" cy="78" rx="8" ry="4.5" />
      <ellipse class="blush" cx="123" cy="78" rx="8" ry="4.5" />

      <g class="eye" class:closed={eyesClosed}>
        <ellipse class="eye-white" cx="89" cy="66" rx="8" ry="9" />
        <circle class="pupil" cx={89 + pupilOffset.x} cy={66 + pupilOffset.y} r="4.4" />
        <circle class="eye-shine" cx={89 + pupilOffset.x - 1.5} cy={66 + pupilOffset.y - 2} r="1.7" />
        <path class="lashes" d="M 81 60 L 78 56 M 86 57 L 85 53" />
        <path class="eyelid" d="M 81 66 Q 89 66 97 66" />
      </g>
      <g class="eye" class:closed={eyesClosed}>
        <ellipse class="eye-white" cx="111" cy="66" rx="8" ry="9" />
        <circle class="pupil" cx={111 + pupilOffset.x} cy={66 + pupilOffset.y} r="4.4" />
        <circle class="eye-shine" cx={111 + pupilOffset.x - 1.5} cy={66 + pupilOffset.y - 2} r="1.7" />
        <path class="lashes" d="M 119 60 L 122 56 M 114 57 L 115 53" />
        <path class="eyelid" d="M 103 66 Q 111 66 119 66" />
      </g>

      <path class="mouth" d={mouthPath} />
      <ellipse class="tear" cx="84" cy="76" rx="3" ry="5" />

      {#if prop === "lookfar"}
        <!-- Hand raised to the brow, shading her eyes -->
        <path class="prop-lookfar" d="M 118 46 Q 138 38 144 50 Q 130 56 118 54 Z" />
      {/if}
    </g>

    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .luna {
    --scale: 1;
    --skin: #f3cba8;
    --skin-dark: #c89870;
    --hair: #5b3a2e;
    --hair-light: #7a5141;
    --dress-light: #b98df0;
    --dress: #8e5fd4;
    --shoe: #ffd166;
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
    animation: breathe 2.6s ease-in-out infinite;
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

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.16);
    filter: blur(1.5px);
  }

  .dress {
    fill: url(#luna-dress);
    stroke: #7348b8;
    stroke-width: 2;
  }
  .collar {
    fill: #fdfbff;
  }
  .head {
    fill: var(--skin);
    stroke: var(--skin-dark);
    stroke-width: 2;
  }
  .head-shine {
    fill: rgba(255, 255, 255, 0.35);
    filter: blur(2px);
  }
  .hair-back,
  .fringe {
    fill: var(--hair);
  }
  .ponytail path {
    fill: var(--hair-light);
    stroke: var(--hair);
    stroke-width: 1.5;
  }
  .ponytail {
    transform-origin: 136px 58px;
    animation: tail-sway 3s ease-in-out infinite;
  }
  @keyframes tail-sway {
    0%,
    100% {
      transform: rotate(-4deg);
    }
    50% {
      transform: rotate(5deg);
    }
  }

  .limb,
  .hand,
  .shin {
    fill: var(--skin);
    stroke: var(--skin-dark);
    stroke-width: 2;
  }
  .shoe {
    fill: var(--shoe);
    stroke: #d9a93f;
    stroke-width: 2;
  }
  .arm-left {
    transform-origin: 68px 108px;
  }
  .arm-right {
    transform-origin: 132px 108px;
  }
  .leg-left {
    transform-origin: 92px 162px;
  }
  .leg-right {
    transform-origin: 108px 162px;
  }

  .eye-white {
    fill: white;
    stroke: var(--skin-dark);
    stroke-width: 1.4;
  }
  .pupil {
    fill: #3a2a22;
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
  .lashes {
    stroke: var(--hair);
    stroke-width: 2;
    stroke-linecap: round;
  }
  .eyelid {
    fill: none;
    stroke: var(--skin-dark);
    stroke-width: 2.6;
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
    stroke: #a8574f;
    stroke-width: 2.8;
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

  .prop-bottle rect {
    fill: #4fc3f7;
    stroke: #2196c9;
    stroke-width: 1.5;
  }
  .prop-bottle path {
    stroke: rgba(255, 255, 255, 0.7);
    stroke-width: 2;
  }
  .prop-lookfar {
    fill: var(--skin);
    stroke: var(--skin-dark);
    stroke-width: 1.5;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(48px, 38px);
  }
  .s2 {
    transform: translate(156px, 62px);
  }

  /* --- A skip, not a march: one leg leads, the body lifts between
         steps, and the ponytail swings a beat behind. --- */
  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.6s ease-in-out infinite,
      skip calc(480ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes skip {
    0%,
    100% {
      transform: translateY(0) rotate(-2deg);
    }
    35% {
      transform: translateY(-13px) rotate(2deg);
    }
    60% {
      transform: translateY(-4px) rotate(0deg);
    }
  }
  .state-walk_in .leg-left,
  .state-walk_out .leg-left {
    animation: skip-leg-a calc(480ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .leg-right,
  .state-walk_out .leg-right {
    animation: skip-leg-b calc(480ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes skip-leg-a {
    0%,
    100% {
      transform: rotate(-24deg);
    }
    40% {
      transform: rotate(16deg);
    }
  }
  @keyframes skip-leg-b {
    0%,
    100% {
      transform: rotate(16deg);
    }
    40% {
      transform: rotate(-24deg);
    }
  }
  .state-walk_in .arm-left,
  .state-walk_out .arm-left {
    animation: skip-arm-a calc(480ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .arm-right,
  .state-walk_out .arm-right {
    animation: skip-arm-b calc(480ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes skip-arm-a {
    0%,
    100% {
      transform: rotate(18deg);
    }
    40% {
      transform: rotate(-18deg);
    }
  }
  @keyframes skip-arm-b {
    0%,
    100% {
      transform: rotate(-18deg);
    }
    40% {
      transform: rotate(18deg);
    }
  }
  .state-walk_in .ponytail,
  .state-walk_out .ponytail {
    animation: tail-bounce calc(480ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes tail-bounce {
    0%,
    100% {
      transform: rotate(-12deg);
    }
    50% {
      transform: rotate(14deg);
    }
  }

  /* --- States --- */
  .state-ask .prop-bottle,
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

  /* A full twirl with both arms thrown up */
  .state-happy .body-svg {
    animation:
      breathe 2.6s ease-in-out infinite,
      twirl 760ms ease-in-out;
    transform-origin: 100px 196px;
  }
  .state-happy .arm-left {
    animation: arms-up-left 420ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
  .state-happy .arm-right {
    animation: arms-up-right 420ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
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
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes twirl {
    0% {
      transform: rotateY(0deg);
    }
    100% {
      transform: rotateY(360deg);
    }
  }
  @keyframes arms-up-left {
    to {
      transform: rotate(-142deg) translate(-4px, 10px);
    }
  }
  @keyframes arms-up-right {
    to {
      transform: rotate(142deg) translate(4px, 10px);
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

  /* Hands tucked behind her back */
  .state-shrug .arm-left {
    transform: rotate(-26deg) translateX(8px);
  }
  .state-shrug .arm-right {
    transform: rotate(26deg) translateX(-8px);
  }

  .state-sad .head-group {
    transform: translateY(6px) rotate(-4deg);
  }
  .state-sad .ponytail {
    animation: none;
    transform: rotate(16deg);
  }
  .state-sad .arm-left,
  .state-sad .arm-right {
    transform: translateY(5px);
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
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(-128deg) translate(4px, 10px);
    }
    50% {
      transform: rotate(-158deg) translate(4px, 10px);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 96px;
  }
  .gesture-hairflick.state-idle .ponytail {
    animation: hairflick 700ms ease-in-out 2;
  }
  .gesture-hairflick.state-idle .head-group {
    animation: head-flick 700ms ease-in-out 2;
  }
  @keyframes hairflick {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-22deg);
    }
  }
  @keyframes head-flick {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(7deg);
    }
  }

  .gesture-rock.state-idle .body-svg {
    animation:
      breathe 2.6s ease-in-out infinite,
      rock 1400ms ease-in-out;
    transform-origin: 100px 196px;
  }
  @keyframes rock {
    0%,
    100% {
      transform: rotate(0deg);
    }
    30% {
      transform: rotate(-5deg);
    }
    70% {
      transform: rotate(5deg);
    }
  }

  /* Finger to the chin, looking up */
  .gesture-think.state-idle .arm-right {
    animation: think-arm 1500ms ease-in-out;
  }
  .gesture-think.state-idle .head-group {
    animation: think-head 1500ms ease-in-out;
  }
  @keyframes think-arm {
    0%,
    100% {
      transform: rotate(0deg);
    }
    35%,
    70% {
      transform: rotate(-108deg) translate(2px, 16px);
    }
  }
  @keyframes think-head {
    0%,
    100% {
      transform: rotate(0deg);
    }
    35%,
    70% {
      transform: rotate(-9deg) translateY(-2px);
    }
  }
</style>
