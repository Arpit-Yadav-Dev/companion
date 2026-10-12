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
    idleGestures: ["twirl", "shimmer", "peek"],
    eyeHeightRatio: 0.45,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 84 112 Q 100 132 116 112",
    shrug: "M 90 116 Q 100 114 110 116",
    sad: "M 90 118 Q 100 110 110 118",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 88 114 Q 100 124 112 114");

  // Twelve rays, evenly spaced — generated rather than hand-written so
  // the spacing stays exact.
  const RAYS = Array.from({ length: 12 }, (_, i) => i * 30);
</script>

<div
  class="sunny state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <radialGradient id="sunny-core" cx="38%" cy="30%" r="74%">
        <stop offset="0%" stop-color="var(--sun-light)" />
        <stop offset="62%" stop-color="var(--sun)" />
        <stop offset="100%" stop-color="var(--sun-deep)" />
      </radialGradient>
    </defs>

    <!-- Floating, so the shadow stays small and soft on the ground -->
    <ellipse class="ground-shadow" cx="100" cy="200" rx="26" ry="5" />

    <g class="float-group">
      <g class="rays">
        {#each RAYS as angle (angle)}
          <path class="ray" d="M 100 26 L 94 46 L 106 46 Z" style="--a: {angle}deg" />
        {/each}
      </g>

      <g class="body-squash">
        <circle class="core" cx="100" cy="104" r="50" />
        <ellipse class="core-shine" cx="80" cy="84" rx="18" ry="13" />

        <ellipse class="blush" cx="68" cy="114" rx="10" ry="6" />
        <ellipse class="blush" cx="132" cy="114" rx="10" ry="6" />

        <g class="eye" class:closed={eyesClosed}>
          <circle class="eye-white" cx="82" cy="96" r="11" />
          <circle class="pupil" cx={82 + pupilOffset.x} cy={96 + pupilOffset.y} r="5.5" />
          <circle class="eye-shine" cx={82 + pupilOffset.x - 2} cy={96 + pupilOffset.y - 2.5} r="2" />
          <path class="eyelid" d="M 71 96 Q 82 96 93 96" />
        </g>
        <g class="eye" class:closed={eyesClosed}>
          <circle class="eye-white" cx="118" cy="96" r="11" />
          <circle class="pupil" cx={118 + pupilOffset.x} cy={96 + pupilOffset.y} r="5.5" />
          <circle class="eye-shine" cx={118 + pupilOffset.x - 2} cy={96 + pupilOffset.y - 2.5} r="2" />
          <path class="eyelid" d="M 107 96 Q 118 96 129 96" />
        </g>

        <path class="mouth" d={mouthPath} />
        <ellipse class="tear" cx="86" cy="108" rx="3" ry="5" />

        {#if prop === "lookfar"}
          <!-- Sunglasses: the sun's own version of shading its eyes -->
          <g class="prop-shades">
            <rect x="66" y="88" width="30" height="18" rx="7" />
            <rect x="104" y="88" width="30" height="18" rx="7" />
            <path d="M 96 95 L 104 95" />
          </g>
        {/if}
      </g>
    </g>

    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .sunny {
    --scale: 1;
    --sun-light: #fff3b0;
    --sun: #ffd23f;
    --sun-deep: #f0a500;
    --ray: #ffc233;
    --blush: #ff9472;

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

  /* Never touches the ground — a gentle hover is the resting state. */
  .float-group {
    animation: hover 3.4s ease-in-out infinite;
    transform-origin: 100px 104px;
  }
  @keyframes hover {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-8px);
    }
  }

  .core {
    fill: url(#sunny-core);
    stroke: var(--sun-deep);
    stroke-width: 2;
  }
  .core-shine {
    fill: rgba(255, 255, 255, 0.5);
    filter: blur(3px);
  }

  .rays {
    transform-origin: 100px 104px;
    animation: ray-spin 18s linear infinite;
  }
  @keyframes ray-spin {
    to {
      transform: rotate(360deg);
    }
  }
  .ray {
    fill: var(--ray);
    transform: rotate(var(--a));
    transform-origin: 100px 104px;
  }

  .eye-white {
    fill: white;
    stroke: var(--sun-deep);
    stroke-width: 1.5;
  }
  .pupil {
    fill: #6b4600;
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
    stroke: var(--sun-deep);
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
    stroke: #a06a00;
    stroke-width: 3.5;
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

  .prop-shades rect {
    fill: #3a3a44;
    stroke: #22222a;
    stroke-width: 1.5;
  }
  .prop-shades path {
    stroke: #22222a;
    stroke-width: 2.5;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #fffbe6;
  }
  .s1 {
    transform: translate(40px, 44px);
  }
  .s2 {
    transform: translate(162px, 70px);
  }

  /* --- Drifts in on the breeze rather than walking: a slow bob with a
         slight tilt, rays spinning faster as it travels. --- */
  .state-walk_in .float-group,
  .state-walk_out .float-group {
    animation: drift calc(1100ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes drift {
    0%,
    100% {
      transform: translateY(0) rotate(-5deg);
    }
    50% {
      transform: translateY(-14px) rotate(5deg);
    }
  }
  .state-walk_in .rays,
  .state-walk_out .rays {
    animation: ray-spin 4s linear infinite;
  }

  /* --- States --- */
  .state-ask .prop-shades {
    animation: prop-bob 1.8s ease-in-out infinite;
  }
  @keyframes prop-bob {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-2px) rotate(-2deg);
    }
  }

  /* Rays fire outward in a burst and the core brightens */
  .state-happy .rays {
    animation:
      ray-spin 2s linear infinite,
      ray-burst 520ms ease-out;
  }
  .state-happy .core {
    animation: core-flare 520ms ease-out;
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
  @keyframes ray-burst {
    0% {
      scale: 1;
    }
    45% {
      scale: 1.3;
    }
    100% {
      scale: 1;
    }
  }
  @keyframes core-flare {
    0%,
    100% {
      filter: brightness(1);
    }
    45% {
      filter: brightness(1.35);
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

  /* Rays wilt and droop — the sun going behind a cloud */
  .state-shrug .rays {
    animation: none;
    transform: scale(0.82);
    opacity: 0.7;
  }
  .state-sad .rays {
    animation: none;
    transform: scale(0.7);
    opacity: 0.5;
  }
  .state-sad .float-group {
    animation: sad-sink 2.6s ease-in-out infinite;
  }
  @keyframes sad-sink {
    0%,
    100% {
      transform: translateY(8px);
    }
    50% {
      transform: translateY(12px);
    }
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

  /* No arms to wave, so the whole body tips side to side instead */
  .state-wave .float-group {
    animation: body-wave 420ms ease-in-out 4;
  }
  @keyframes body-wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    25% {
      transform: rotate(-14deg);
    }
    75% {
      transform: rotate(14deg);
    }
  }

  /* --- Idle gestures --- */
  .gesture-twirl.state-idle .float-group {
    animation: twirl 1500ms ease-in-out;
  }
  @keyframes twirl {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(360deg);
    }
  }

  .gesture-shimmer.state-idle .core {
    animation: shimmer 420ms ease-in-out 3;
  }
  @keyframes shimmer {
    0%,
    100% {
      filter: brightness(1);
    }
    50% {
      filter: brightness(1.25);
    }
  }

  .gesture-peek.state-idle .body-squash {
    animation: peek 1300ms ease-in-out;
    transform-origin: 100px 104px;
  }
  @keyframes peek {
    0%,
    100% {
      transform: rotate(0deg);
    }
    35% {
      transform: rotate(-16deg);
    }
    70% {
      transform: rotate(16deg);
    }
  }
</style>
