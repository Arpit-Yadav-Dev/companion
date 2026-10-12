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
      <radialGradient id="mochi-body" cx="40%" cy="30%" r="76%">
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

    <ellipse class="ground-shadow" cx="100" cy="200" rx="44" ry="8" />

    <!-- Sitting pose: a tall teardrop body that tapers to a narrow chest,
         with the tail curled around the base. Deliberately unlike the
         wide four-legged stance of the dog characters. -->
    <g class="tail-root">
      <path
        class="tail"
        d="M 68 192 Q 26 192 30 160 Q 33 138 52 142"
      />
    </g>

    <g class="body-squash">
      <path
        class="body"
        d="M 100 96
           C 128 96 142 132 144 164
           C 146 188 128 198 100 198
           C 72 198 54 188 56 164
           C 58 132 72 96 100 96 Z"
      />
      <!-- Pale chest bib down the front -->
      <path class="bib" d="M 100 126 C 114 126 120 152 118 176 C 116 190 84 190 82 176 C 80 152 86 126 100 126 Z" />
    </g>

    <!-- Front paws tucked together at the base, cat-sitting style -->
    <g class="paws">
      <ellipse class="paw paw-left" cx="84" cy="192" rx="14" ry="8" />
      <ellipse class="paw paw-right" cx="116" cy="192" rx="14" ry="8" />
      <path class="toe" d="M 79 190 L 79 194 M 84 189 L 84 194 M 89 190 L 89 194" />
      <path class="toe" d="M 111 190 L 111 194 M 116 189 L 116 194 M 121 190 L 121 194" />
    </g>

    <g class="head-group">
      <!-- Wide, low-set triangular ears -->
      <path class="ear ear-left" d="M 62 78 L 54 36 L 94 62 Z" />
      <path class="ear ear-right" d="M 138 78 L 146 36 L 106 62 Z" />
      <path class="ear-inner" d="M 67 74 L 61 46 L 87 64 Z" />
      <path class="ear-inner" d="M 133 74 L 139 46 L 113 64 Z" />

      <!-- Rounder, wider head than the dogs' circle -->
      <ellipse class="head" cx="100" cy="90" rx="46" ry="40" />
      <ellipse class="head-shine" cx="80" cy="72" rx="15" ry="10" />

      <!-- Cheek tufts give the wide cat face shape -->
      <path class="cheek" d="M 56 94 Q 46 104 58 112" />
      <path class="cheek" d="M 144 94 Q 154 104 142 112" />

      <ellipse class="muzzle" cx="100" cy="110" rx="24" ry="15" />

      <ellipse class="blush" cx="68" cy="104" rx="9" ry="5" />
      <ellipse class="blush" cx="132" cy="104" rx="9" ry="5" />

      <!-- Tall almond eyes with slit pupils — the clearest cat cue -->
      <g class="eye" class:closed={eyesClosed}>
        <ellipse class="eye-white" cx="82" cy="86" rx="11" ry="12" />
        <ellipse class="pupil" cx={82 + pupilOffset.x} cy={86 + pupilOffset.y} rx="3.4" ry="10" />
        <circle class="eye-shine" cx={82 + pupilOffset.x - 2} cy={86 + pupilOffset.y - 4} r="2" />
        <path class="eyelid" d="M 71 86 Q 82 86 93 86" />
      </g>
      <g class="eye" class:closed={eyesClosed}>
        <ellipse class="eye-white" cx="118" cy="86" rx="11" ry="12" />
        <ellipse class="pupil" cx={118 + pupilOffset.x} cy={86 + pupilOffset.y} rx="3.4" ry="10" />
        <circle class="eye-shine" cx={118 + pupilOffset.x - 2} cy={86 + pupilOffset.y - 4} r="2" />
        <path class="eyelid" d="M 107 86 Q 118 86 129 86" />
      </g>

      <path class="nose" d="M 93 104 L 107 104 L 100 112 Z" />
      <path class="mouth" d={mouthPath} />

      <g class="whiskers">
        <path d="M 72 112 L 42 106" />
        <path d="M 72 118 L 44 120" />
        <path d="M 128 112 L 158 106" />
        <path d="M 128 118 L 156 120" />
      </g>

      <ellipse class="tear" cx="86" cy="98" rx="3" ry="5" />

      {#if prop === "lookfar"}
        <ellipse class="prop-lookfar" cx="128" cy="76" rx="11" ry="7" />
      {:else if prop !== "stretch" && prop !== "posture"}
        <g class="prop-saucer">
          <ellipse cx="150" cy="150" rx="17" ry="6" />
          <path d="M 135 148 Q 150 159 165 148" />
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
    /* Warm ginger tabby, against Scout's cool grey husky. */
    --fur-light: #f8d09b;
    --fur: #e9a65a;
    --fur-shadow: #c47c34;
    --fur-dark: #8a5421;
    --bib: #fdf3e4;
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
  .bib {
    fill: var(--bib);
    opacity: 0.9;
  }
  .head {
    fill: url(#mochi-head);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .head-shine {
    fill: rgba(255, 255, 255, 0.4);
    filter: blur(2px);
  }
  .muzzle {
    fill: var(--bib);
  }
  .cheek {
    fill: none;
    stroke: var(--fur-shadow);
    stroke-width: 2.5;
    stroke-linecap: round;
    opacity: 0.7;
  }

  .paw {
    fill: var(--fur-light);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .toe {
    stroke: var(--fur-shadow);
    stroke-width: 1.4;
    stroke-linecap: round;
    opacity: 0.8;
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
    stroke-width: 14;
    stroke-linecap: round;
  }
  .tail-root {
    transform-origin: 68px 192px;
    animation: tail-idle 3.4s ease-in-out infinite;
  }
  @keyframes tail-idle {
    0%,
    100% {
      transform: rotate(-3deg);
    }
    50% {
      transform: rotate(5deg);
    }
  }

  .eye-white {
    fill: white;
    stroke: var(--fur-dark);
    stroke-width: 1.5;
  }
  .pupil {
    fill: #23331f;
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
    stroke: rgba(80, 55, 30, 0.42);
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
    fill: var(--fur-light);
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
    transform: translate(44px, 52px);
  }
  .s2 {
    transform: translate(158px, 74px);
  }

  /* A sitting cat has no walk cycle to run — it hops, body and all,
     which also keeps it visually distinct from the dogs' leg-over-leg
     gait at a glance. */
  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.8s ease-in-out infinite,
      hop calc(420ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes hop {
    0%,
    100% {
      transform: translateY(0) scaleY(1);
    }
    30% {
      transform: translateY(-12px) scaleY(1.05);
    }
    60% {
      transform: translateY(0) scaleY(0.94);
    }
  }
  .state-walk_in .tail-root,
  .state-walk_out .tail-root {
    animation: tail-walk calc(420ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes tail-walk {
    0%,
    100% {
      transform: rotate(-10deg);
    }
    50% {
      transform: rotate(12deg);
    }
  }

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
    transform-origin: 100px 198px;
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

  .state-shrug .paws {
    animation: shrug-paws 600ms ease-out both;
  }
  @keyframes shrug-paws {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(-6px);
    }
  }

  .state-sad .ear {
    transform: translateY(7px) scale(0.93);
    transform-origin: 100px 78px;
  }
  .state-sad .tail-root {
    transform: rotate(-18deg);
  }
  .state-sad .tear {
    opacity: 1;
    animation: tear-drop 1.4s ease-in infinite;
  }
  .state-sad .head-group {
    transform: translateY(5px) rotate(-3deg);
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

  .state-wave .paw-right {
    animation: wave 520ms ease-in-out 3;
    transform-origin: 116px 192px;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg) translateY(0);
    }
    50% {
      transform: rotate(-40deg) translateY(-18px);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 130px;
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
      transform: rotate(-16deg) translateY(10px);
    }
  }
  .gesture-groom.state-idle .paw-left {
    animation: gesture-groom-paw 1400ms ease-in-out;
    transform-origin: 84px 192px;
  }
  @keyframes gesture-groom-paw {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    35%,
    65% {
      transform: translateY(-34px) rotate(-18deg);
    }
  }

  .gesture-earflick.state-idle .ear-right {
    animation: gesture-earflick 620ms ease-in-out 2;
    transform-origin: 138px 78px;
  }
  @keyframes gesture-earflick {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-12deg);
    }
  }

  .gesture-stretch-back.state-idle .body-squash {
    animation: gesture-stretch 1400ms ease-in-out;
    transform-origin: 100px 198px;
  }
  @keyframes gesture-stretch {
    0%,
    100% {
      transform: scale(1, 1);
    }
    40%,
    60% {
      transform: scale(1.08, 0.93) translateY(4px);
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
    animation: puff-rise 430ms ease-out infinite;
  }
  .p1 {
    left: 10px;
    animation-delay: 0ms;
  }
  .p2 {
    left: 31px;
    animation-delay: 140ms;
  }
  .p3 {
    left: 52px;
    animation-delay: 280ms;
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
