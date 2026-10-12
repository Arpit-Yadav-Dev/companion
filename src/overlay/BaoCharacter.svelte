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
    idleGestures: ["munch", "sway", "yawn"],
    eyeHeightRatio: 0.34,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 88 118 Q 100 132 112 118",
    shrug: "M 92 120 Q 100 118 108 120",
    sad: "M 92 122 Q 100 115 108 122",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 92 118 Q 100 125 108 118");
</script>

<div
  class="bao state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <ellipse class="ground-shadow" cx="100" cy="200" rx="48" ry="8" />

    <g class="roll-group">
      <!-- Back legs splayed out in front, panda-sitting -->
      <ellipse class="limb leg-left" cx="72" cy="186" rx="20" ry="13" />
      <ellipse class="limb leg-right" cx="128" cy="186" rx="20" ry="13" />

      <g class="body-squash">
        <!-- Almost a circle: a panda is a ball with a face. -->
        <ellipse class="body" cx="100" cy="146" rx="50" ry="46" />
        <!-- Black shoulder band that wraps the chest -->
        <path
          class="band"
          d="M 60 124 Q 100 112 140 124 Q 142 142 138 152 Q 100 140 62 152 Q 58 142 60 124 Z"
        />
      </g>

      <!-- Arms: short, dark, and the ones that clap -->
      <g class="arm arm-left">
        <ellipse class="limb-dark" cx="56" cy="150" rx="15" ry="20" />
      </g>
      <g class="arm arm-right">
        <ellipse class="limb-dark" cx="144" cy="150" rx="15" ry="20" />
      </g>

      <g class="head-group">
        <!-- Round black ears sitting high on the skull -->
        <circle class="ear ear-left" cx="68" cy="56" r="17" />
        <circle class="ear ear-right" cx="132" cy="56" r="17" />

        <ellipse class="head" cx="100" cy="82" rx="44" ry="40" />
        <ellipse class="head-shine" cx="82" cy="66" rx="14" ry="10" />

        <!-- The signature eye patches, angled inward -->
        <ellipse class="patch patch-left" cx="80" cy="84" rx="16" ry="19" />
        <ellipse class="patch patch-right" cx="120" cy="84" rx="16" ry="19" />

        <g class="eye" class:closed={eyesClosed}>
          <circle class="eye-white" cx="80" cy="84" r="8.5" />
          <circle class="pupil" cx={80 + pupilOffset.x} cy={84 + pupilOffset.y} r="4.5" />
          <circle class="eye-shine" cx={80 + pupilOffset.x - 1.6} cy={84 + pupilOffset.y - 2} r="1.8" />
          <path class="eyelid" d="M 71 84 Q 80 84 89 84" />
        </g>
        <g class="eye" class:closed={eyesClosed}>
          <circle class="eye-white" cx="120" cy="84" r="8.5" />
          <circle class="pupil" cx={120 + pupilOffset.x} cy={84 + pupilOffset.y} r="4.5" />
          <circle class="eye-shine" cx={120 + pupilOffset.x - 1.6} cy={84 + pupilOffset.y - 2} r="1.8" />
          <path class="eyelid" d="M 111 84 Q 120 84 129 84" />
        </g>

        <ellipse class="muzzle" cx="100" cy="110" rx="22" ry="15" />
        <ellipse class="nose" cx="100" cy="106" rx="8" ry="6" />
        <path class="mouth" d={mouthPath} />

        <ellipse class="tear" cx="86" cy="96" rx="3" ry="5" />

        {#if prop === "lookfar"}
          <ellipse class="prop-lookfar" cx="132" cy="70" rx="12" ry="8" />
        {/if}
      </g>

      <!-- Bamboo held in the left paw, grounded against the body -->
      {#if prop !== "lookfar"}
        <g class="prop-bamboo">
          <rect x="44" y="120" width="9" height="62" rx="4" />
          <path d="M 44 140 L 53 140 M 44 160 L 53 160" />
          <path class="leaf" d="M 48 120 Q 30 110 26 122 Q 38 128 48 124 Z" />
          <path class="leaf" d="M 49 124 Q 36 136 46 142 Q 52 134 49 124 Z" />
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
  .bao {
    --scale: 1;
    --white: #fbfbf8;
    --white-shade: #ded9d0;
    --black: #2b2b31;
    --black-soft: #3d3d45;
    --muzzle: #ffffff;
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
    animation: breathe 3.2s ease-in-out infinite;
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

  .body,
  .head {
    fill: var(--white);
    stroke: var(--white-shade);
    stroke-width: 2;
  }
  .head-shine {
    fill: rgba(255, 255, 255, 0.6);
    filter: blur(2px);
  }
  .band {
    fill: var(--black);
  }
  .limb {
    fill: var(--white);
    stroke: var(--white-shade);
    stroke-width: 2;
  }
  .limb-dark {
    fill: var(--black);
  }
  .ear {
    fill: var(--black);
  }
  .patch {
    fill: var(--black);
    transform: rotate(-12deg);
    transform-origin: center;
  }
  .patch-right {
    transform: rotate(12deg);
    transform-origin: center;
  }
  .muzzle {
    fill: var(--muzzle);
  }
  .nose {
    fill: var(--black);
  }

  .arm-left {
    transform-origin: 62px 136px;
  }
  .arm-right {
    transform-origin: 138px 136px;
  }

  .eye-white {
    fill: white;
  }
  .pupil {
    fill: #1a1a1f;
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
    stroke: var(--white);
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

  .mouth {
    fill: none;
    stroke: var(--black);
    stroke-width: 3;
    stroke-linecap: round;
  }
  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-bamboo rect {
    fill: #8bc34a;
    stroke: #5d9130;
    stroke-width: 1.5;
  }
  .prop-bamboo path {
    fill: none;
    stroke: #5d9130;
    stroke-width: 1.5;
  }
  .prop-bamboo .leaf {
    fill: #7cb342;
    stroke: #5d9130;
    stroke-width: 1.2;
  }
  .prop-lookfar {
    fill: var(--black);
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(42px, 40px);
  }
  .s2 {
    transform: translate(160px, 66px);
  }

  /* --- A panda is round, so it rolls rather than walks: the whole
         character tumbles forward, squashing on each contact. --- */
  .roll-group {
    transform-origin: 100px 160px;
  }
  .state-walk_in .roll-group,
  .state-walk_out .roll-group {
    animation: tumble calc(900ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes tumble {
    0% {
      transform: rotate(-14deg) translateY(0);
    }
    25% {
      transform: rotate(0deg) translateY(-8px);
    }
    50% {
      transform: rotate(14deg) translateY(0);
    }
    75% {
      transform: rotate(0deg) translateY(-8px);
    }
    100% {
      transform: rotate(-14deg) translateY(0);
    }
  }
  .state-walk_in .body-squash,
  .state-walk_out .body-squash {
    animation: roll-squash calc(900ms / var(--anim-speed, 1)) ease-in-out infinite;
    transform-origin: 100px 192px;
  }
  @keyframes roll-squash {
    0%,
    50%,
    100% {
      transform: scale(1.05, 0.95);
    }
    25%,
    75% {
      transform: scale(0.97, 1.03);
    }
  }

  /* --- States --- */
  .state-ask .prop-bamboo,
  .state-ask .prop-lookfar {
    animation: prop-bob 1.9s ease-in-out infinite;
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

  /* Paws clap together, belly wobbles behind the beat */
  .state-happy .arm-left {
    animation: clap-left 300ms ease-in-out infinite;
  }
  .state-happy .arm-right {
    animation: clap-right 300ms ease-in-out infinite;
  }
  .state-happy .body-squash {
    animation: belly-wobble 300ms ease-in-out infinite;
    transform-origin: 100px 192px;
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
  @keyframes clap-left {
    0%,
    100% {
      transform: translateX(0) rotate(0deg);
    }
    50% {
      transform: translateX(32px) rotate(-16deg);
    }
  }
  @keyframes clap-right {
    0%,
    100% {
      transform: translateX(0) rotate(0deg);
    }
    50% {
      transform: translateX(-32px) rotate(16deg);
    }
  }
  @keyframes belly-wobble {
    0%,
    100% {
      transform: scale(1, 1);
    }
    50% {
      transform: scale(1.04, 0.96);
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

  /* Covers its eyes with one paw — bashful rather than annoyed */
  .state-shrug .arm-right {
    animation: cover-eyes 600ms ease-out both;
  }
  @keyframes cover-eyes {
    to {
      transform: translate(-28px, -52px) rotate(-22deg);
    }
  }

  .state-sad .head-group {
    transform: translateY(6px) rotate(-3deg);
  }
  .state-sad .arm-left,
  .state-sad .arm-right {
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
    animation: wave 560ms ease-in-out 3;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg) translateY(0);
    }
    50% {
      transform: rotate(-34deg) translateY(-26px);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 120px;
  }
  .gesture-munch.state-idle .mouth {
    animation: munch 340ms ease-in-out 4;
    transform-origin: 100px 120px;
  }
  @keyframes munch {
    0%,
    100% {
      transform: scaleY(1);
    }
    50% {
      transform: scaleY(0.4) translateY(2px);
    }
  }

  .gesture-sway.state-idle .roll-group {
    animation: sway 1600ms ease-in-out;
  }
  @keyframes sway {
    0%,
    100% {
      transform: rotate(0deg);
    }
    30% {
      transform: rotate(-6deg);
    }
    70% {
      transform: rotate(6deg);
    }
  }

  .gesture-yawn.state-idle .mouth {
    animation: yawn 1400ms ease-in-out;
    transform-origin: 100px 120px;
  }
  .gesture-yawn.state-idle .head-group {
    animation: yawn-tilt 1400ms ease-in-out;
  }
  @keyframes yawn {
    0%,
    100% {
      transform: scaleY(1);
    }
    40%,
    60% {
      transform: scaleY(2.6) translateY(3px);
    }
  }
  @keyframes yawn-tilt {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-8deg) translateY(-3px);
    }
  }
</style>
