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
    idleGestures: ["sway", "unfurl", "shiver"],
    eyeHeightRatio: 0.52,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 90 136 Q 100 150 110 136",
    shrug: "M 93 138 Q 100 136 107 138",
    sad: "M 93 140 Q 100 133 107 140",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 93 137 Q 100 144 107 137");
</script>

<div
  class="sprout state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <linearGradient id="sprout-pot" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="var(--pot-light)" />
        <stop offset="100%" stop-color="var(--pot)" />
      </linearGradient>
      <radialGradient id="sprout-leaf" cx="35%" cy="30%" r="80%">
        <stop offset="0%" stop-color="var(--leaf-light)" />
        <stop offset="100%" stop-color="var(--leaf)" />
      </radialGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="202" rx="38" ry="7" />

    <g class="whole-plant">
      <!-- Leaves fan out from the stem; each one sways on its own hinge -->
      <g class="leaf leaf-far-left">
        <path d="M 88 92 Q 48 70 38 96 Q 62 112 88 100 Z" />
        <path class="vein" d="M 86 96 Q 62 92 44 96" />
      </g>
      <g class="leaf leaf-far-right">
        <path d="M 112 92 Q 152 70 162 96 Q 138 112 112 100 Z" />
        <path class="vein" d="M 114 96 Q 138 92 156 96" />
      </g>
      <g class="leaf leaf-left">
        <path d="M 92 76 Q 58 42 44 66 Q 66 92 94 86 Z" />
        <path class="vein" d="M 90 82 Q 68 72 50 64" />
      </g>
      <g class="leaf leaf-right">
        <path d="M 108 76 Q 142 42 156 66 Q 134 92 106 86 Z" />
        <path class="vein" d="M 110 82 Q 132 72 150 64" />
      </g>
      <!-- A top shoot with a bud that blooms on yes -->
      <g class="leaf leaf-top">
        <path d="M 100 72 Q 86 34 100 22 Q 114 34 100 72 Z" />
      </g>
      <g class="flower">
        <circle class="petal" cx="100" cy="18" r="9" />
        <circle class="petal" cx="112" cy="26" r="9" />
        <circle class="petal" cx="88" cy="26" r="9" />
        <circle class="petal" cx="105" cy="38" r="9" />
        <circle class="petal" cx="95" cy="38" r="9" />
        <circle class="flower-core" cx="100" cy="28" r="7" />
      </g>

      <!-- Stem runs behind the pot rim -->
      <rect class="stem" x="95" y="86" width="10" height="56" rx="5" />

      <g class="pot-group">
        <!-- Terracotta pot: the whole lower body, no legs -->
        <path class="pot" d="M 68 140 L 132 140 L 124 194 Q 100 200 76 194 Z" />
        <rect class="rim" x="64" y="132" width="72" height="14" rx="5" />
        <ellipse class="soil" cx="100" cy="139" rx="30" ry="5" />

        <!-- The face lives on the pot, where it's readable -->
        <ellipse class="blush" cx="78" cy="168" rx="8" ry="5" />
        <ellipse class="blush" cx="122" cy="168" rx="8" ry="5" />

        <g class="eye" class:closed={eyesClosed}>
          <circle class="eye-white" cx="86" cy="160" r="9" />
          <circle class="pupil" cx={86 + pupilOffset.x} cy={160 + pupilOffset.y} r="4.6" />
          <circle class="eye-shine" cx={86 + pupilOffset.x - 1.6} cy={160 + pupilOffset.y - 2} r="1.8" />
          <path class="eyelid" d="M 77 160 Q 86 160 95 160" />
        </g>
        <g class="eye" class:closed={eyesClosed}>
          <circle class="eye-white" cx="114" cy="160" r="9" />
          <circle class="pupil" cx={114 + pupilOffset.x} cy={160 + pupilOffset.y} r="4.6" />
          <circle class="eye-shine" cx={114 + pupilOffset.x - 1.6} cy={160 + pupilOffset.y - 2} r="1.8" />
          <path class="eyelid" d="M 105 160 Q 114 160 123 160" />
        </g>

        <path class="mouth" d={mouthPath} />
        <ellipse class="tear" cx="82" cy="170" rx="3" ry="5" />
      </g>

      {#if prop === "lookfar"}
        <!-- A leaf curls up like a hand over the brow -->
        <path class="prop-lookfar" d="M 120 142 Q 144 132 150 146 Q 134 154 120 150 Z" />
      {:else if prop !== "stretch" && prop !== "posture"}
        <!-- Watering can, resting on the ground beside the pot -->
        <g class="prop-can">
          <rect x="142" y="166" width="24" height="24" rx="5" />
          <path class="spout" d="M 166 172 L 182 164 L 184 170 L 168 180 Z" />
          <path class="handle" d="M 146 166 Q 154 152 162 166" />
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
  .sprout {
    --scale: 1;
    --leaf-light: #8fd86b;
    --leaf: #52a83c;
    --leaf-dark: #357a26;
    --pot-light: #e08a5a;
    --pot: #c26840;
    --pot-dark: #93492b;
    --soil: #4b3426;
    --petal: #ffd9ec;
    --petal-core: #ffcf4d;
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
    transform-origin: 50% 100%;
  }

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.16);
    filter: blur(1.5px);
  }

  .pot {
    fill: url(#sprout-pot);
    stroke: var(--pot-dark);
    stroke-width: 2;
  }
  .rim {
    fill: var(--pot-light);
    stroke: var(--pot-dark);
    stroke-width: 2;
  }
  .soil {
    fill: var(--soil);
  }
  .stem {
    fill: var(--leaf-dark);
  }

  .leaf path {
    fill: url(#sprout-leaf);
    stroke: var(--leaf-dark);
    stroke-width: 2;
  }
  .leaf .vein {
    fill: none;
    stroke: var(--leaf-dark);
    stroke-width: 1.5;
    opacity: 0.55;
  }
  /* Each leaf hinges where it meets the stem, so they can sway
     independently like real foliage. */
  .leaf-left {
    transform-origin: 94px 84px;
  }
  .leaf-right {
    transform-origin: 106px 84px;
  }
  .leaf-far-left {
    transform-origin: 90px 98px;
  }
  .leaf-far-right {
    transform-origin: 110px 98px;
  }
  .leaf-top {
    transform-origin: 100px 72px;
  }

  /* The bud stays closed until the happy state opens it. */
  .flower {
    opacity: 0;
    transform: scale(0.3);
    transform-origin: 100px 28px;
    transition:
      opacity 240ms ease,
      transform 240ms ease;
  }
  .petal {
    fill: var(--petal);
    stroke: #e8a9c6;
    stroke-width: 1.2;
  }
  .flower-core {
    fill: var(--petal-core);
  }

  .eye-white {
    fill: white;
    stroke: var(--pot-dark);
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
  .eyelid {
    fill: none;
    stroke: var(--pot-dark);
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
    stroke: #7a3f24;
    stroke-width: 2.8;
    stroke-linecap: round;
  }
  .blush {
    fill: var(--blush);
    opacity: 0.45;
  }
  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-can rect {
    fill: #9fd3e8;
    stroke: #5f9cb5;
    stroke-width: 1.5;
  }
  .prop-can .spout,
  .prop-can .handle {
    fill: #9fd3e8;
    stroke: #5f9cb5;
    stroke-width: 1.5;
  }
  .prop-can .handle {
    fill: none;
    stroke-width: 3;
  }
  .prop-lookfar {
    fill: url(#sprout-leaf);
    stroke: var(--leaf-dark);
    stroke-width: 1.5;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(42px, 34px);
  }
  .s2 {
    transform: translate(158px, 58px);
  }

  /* Resting: leaves drift as if in a draught. */
  .leaf-left,
  .leaf-far-left {
    animation: sway-a 4s ease-in-out infinite;
  }
  .leaf-right,
  .leaf-far-right {
    animation: sway-b 4.4s ease-in-out infinite;
  }
  .leaf-top {
    animation: sway-a 3.6s ease-in-out infinite;
  }
  @keyframes sway-a {
    0%,
    100% {
      transform: rotate(-3deg);
    }
    50% {
      transform: rotate(4deg);
    }
  }
  @keyframes sway-b {
    0%,
    100% {
      transform: rotate(3deg);
    }
    50% {
      transform: rotate(-4deg);
    }
  }

  /* --- No legs, so the pot itself hops, and the leaves lag behind on
         each landing like something top-heavy. --- */
  .whole-plant {
    transform-origin: 100px 196px;
  }
  .state-walk_in .whole-plant,
  .state-walk_out .whole-plant {
    animation: pot-hop calc(560ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes pot-hop {
    0%,
    100% {
      transform: translateY(0) scale(1.06, 0.94);
    }
    30% {
      transform: translateY(-20px) scale(0.95, 1.06);
    }
    65% {
      transform: translateY(-4px) scale(1, 1);
    }
  }
  .state-walk_in .leaf-left,
  .state-walk_out .leaf-left,
  .state-walk_in .leaf-far-left,
  .state-walk_out .leaf-far-left {
    animation: leaf-lag-a calc(560ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .leaf-right,
  .state-walk_out .leaf-right,
  .state-walk_in .leaf-far-right,
  .state-walk_out .leaf-far-right {
    animation: leaf-lag-b calc(560ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes leaf-lag-a {
    0%,
    100% {
      transform: rotate(14deg);
    }
    40% {
      transform: rotate(-16deg);
    }
  }
  @keyframes leaf-lag-b {
    0%,
    100% {
      transform: rotate(-14deg);
    }
    40% {
      transform: rotate(16deg);
    }
  }

  /* --- States --- */
  .state-ask .prop-can,
  .state-ask .prop-lookfar {
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

  /* The payoff: the bud blooms and the leaves spread wide */
  .state-happy .flower {
    opacity: 1;
    transform: scale(1);
    animation: bloom 620ms cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .state-happy .leaf-left {
    animation: spread-left 420ms ease-out both;
  }
  .state-happy .leaf-right {
    animation: spread-right 420ms ease-out both;
  }
  .state-happy .whole-plant {
    animation: happy-bob 420ms ease-in-out infinite;
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
  @keyframes bloom {
    0% {
      transform: scale(0.2) rotate(-40deg);
    }
    100% {
      transform: scale(1) rotate(0deg);
    }
  }
  @keyframes spread-left {
    to {
      transform: rotate(-20deg);
    }
  }
  @keyframes spread-right {
    to {
      transform: rotate(20deg);
    }
  }
  @keyframes happy-bob {
    0%,
    100% {
      transform: translateY(0) scale(1, 1);
    }
    50% {
      transform: translateY(-7px) scale(0.98, 1.02);
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

  /* Leaves droop — a plant's version of a shrug */
  .state-shrug .leaf-left {
    transform: rotate(26deg);
  }
  .state-shrug .leaf-right {
    transform: rotate(-26deg);
  }
  .state-shrug .leaf-far-left,
  .state-shrug .leaf-far-right,
  .state-shrug .leaf-top {
    animation: none;
  }

  /* Fully wilted */
  .state-sad .leaf-left {
    animation: none;
    transform: rotate(42deg) translateY(8px);
  }
  .state-sad .leaf-right {
    animation: none;
    transform: rotate(-42deg) translateY(8px);
  }
  .state-sad .leaf-far-left {
    animation: none;
    transform: rotate(30deg) translateY(6px);
  }
  .state-sad .leaf-far-right {
    animation: none;
    transform: rotate(-30deg) translateY(6px);
  }
  .state-sad .leaf-top {
    animation: none;
    transform: rotate(18deg);
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

  /* One leaf waves like an arm */
  .state-wave .leaf-right {
    animation: wave 520ms ease-in-out 3;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(-8deg);
    }
    50% {
      transform: rotate(-38deg);
    }
  }

  /* --- Idle gestures --- */
  .gesture-sway.state-idle .whole-plant {
    animation: gesture-sway 1600ms ease-in-out;
  }
  @keyframes gesture-sway {
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

  .gesture-unfurl.state-idle .leaf-top {
    animation: unfurl 1300ms ease-in-out;
  }
  @keyframes unfurl {
    0%,
    100% {
      transform: rotate(0deg) scaleY(1);
    }
    45% {
      transform: rotate(-8deg) scaleY(1.18);
    }
  }

  .gesture-shiver.state-idle .leaf-left,
  .gesture-shiver.state-idle .leaf-right,
  .gesture-shiver.state-idle .leaf-far-left,
  .gesture-shiver.state-idle .leaf-far-right {
    animation: shiver 140ms ease-in-out 8;
  }
  @keyframes shiver {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(3deg);
    }
  }
</style>
