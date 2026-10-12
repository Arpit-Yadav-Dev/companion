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
    idleGestures: ["bounce", "neckroll", "jab"],
    eyeHeightRatio: 0.24,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 88 70 Q 100 82 112 70",
    shrug: "M 92 73 Q 100 71 108 73",
    sad: "M 92 75 Q 100 69 108 75",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 92 71 Q 100 77 108 71");
</script>

<div
  class="flex state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <linearGradient id="flex-vest" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="var(--vest-light)" />
        <stop offset="100%" stop-color="var(--vest)" />
      </linearGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="204" rx="40" ry="7" />

    <!-- Legs: shorts over sturdy legs, trainers on the ground -->
    <g class="leg leg-left">
      <rect class="shorts" x="80" y="150" width="19" height="26" rx="5" />
      <rect class="shin" x="83" y="172" width="13" height="24" rx="6" />
      <ellipse class="shoe" cx="88" cy="198" rx="14" ry="7" />
    </g>
    <g class="leg leg-right">
      <rect class="shorts" x="101" y="150" width="19" height="26" rx="5" />
      <rect class="shin" x="104" y="172" width="13" height="24" rx="6" />
      <ellipse class="shoe" cx="112" cy="198" rx="14" ry="7" />
    </g>

    <g class="body-squash">
      <!-- Broad shoulders tapering to the waist — a V-shaped torso is
           the whole read here, versus the round animal bodies. -->
      <path
        class="torso"
        d="M 100 92
           C 126 92 140 102 142 118
           C 143 136 130 156 100 156
           C 70 156 57 136 58 118
           C 60 102 74 92 100 92 Z"
      />
      <path class="vest" d="M 100 96 C 116 96 124 104 125 118 C 126 134 118 152 100 152 C 82 152 74 134 75 118 C 76 104 84 96 100 96 Z" />

      <!-- Arms, hinged at the shoulder so they can pump and flex -->
      <g class="arm arm-left">
        <rect class="upper-arm" x="46" y="104" width="18" height="30" rx="9" />
        <ellipse class="bicep" cx="55" cy="114" rx="12" ry="11" />
        <rect class="forearm" x="48" y="130" width="15" height="26" rx="7.5" />
        <circle class="fist" cx="55" cy="158" r="9" />
      </g>
      <g class="arm arm-right">
        <rect class="upper-arm" x="136" y="104" width="18" height="30" rx="9" />
        <ellipse class="bicep" cx="145" cy="114" rx="12" ry="11" />
        <rect class="forearm" x="137" y="130" width="15" height="26" rx="7.5" />
        <circle class="fist" cx="145" cy="158" r="9" />

        {#if prop === "stretch" || prop === "posture"}
          <!-- Towel draped over the fist -->
          <path class="prop-towel" d="M 137 158 Q 150 162 152 182 Q 143 184 138 170 Z" />
        {:else if prop !== "lookfar"}
          <!-- Shaker bottle, gripped in the fist -->
          <g class="prop-shaker">
            <rect x="136" y="160" width="18" height="32" rx="4" />
            <rect x="139" y="153" width="12" height="9" rx="2.5" />
            <path d="M 138 174 L 152 174" />
          </g>
        {/if}
      </g>
    </g>

    <g class="head-group">
      <!-- Sweatband -->
      <path class="band" d="M 72 44 Q 100 34 128 44 L 128 54 Q 100 44 72 54 Z" />
      <ellipse class="head" cx="100" cy="62" rx="30" ry="32" />
      <ellipse class="head-shine" cx="88" cy="48" rx="10" ry="8" />

      <ellipse class="blush" cx="76" cy="72" rx="8" ry="4.5" />
      <ellipse class="blush" cx="124" cy="72" rx="8" ry="4.5" />

      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-white" cx="89" cy="58" r="8" />
        <circle class="pupil" cx={89 + pupilOffset.x} cy={58 + pupilOffset.y} r="4" />
        <circle class="eye-shine" cx={89 + pupilOffset.x - 1.4} cy={58 + pupilOffset.y - 1.8} r="1.5" />
        <path class="eyelid" d="M 81 58 Q 89 58 97 58" />
      </g>
      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-white" cx="111" cy="58" r="8" />
        <circle class="pupil" cx={111 + pupilOffset.x} cy={58 + pupilOffset.y} r="4" />
        <circle class="eye-shine" cx={111 + pupilOffset.x - 1.4} cy={58 + pupilOffset.y - 1.8} r="1.5" />
        <path class="eyelid" d="M 103 58 Q 111 58 119 58" />
      </g>

      <path class="brow brow-left" d="M 82 48 Q 89 44 96 48" />
      <path class="brow brow-right" d="M 104 48 Q 111 44 118 48" />

      <path class="mouth" d={mouthPath} />
      <ellipse class="tear" cx="84" cy="68" rx="3" ry="5" />

      {#if prop === "lookfar"}
        <!-- Hand shading the eyes, resting on the brow -->
        <path class="prop-lookfar" d="M 118 40 Q 140 32 146 44 Q 132 50 118 48 Z" />
      {/if}

      <!-- A bead of sweat, because of course -->
      <path class="sweat" d="M 132 44 Q 136 52 132 56 Q 128 52 132 44 Z" />
    </g>



    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .flex {
    --scale: 1;
    --skin: #e7b188;
    --skin-dark: #b9815b;
    --vest-light: #4fc3f7;
    --vest: #2196c9;
    --shorts: #2f3b4a;
    --shoe: #f5f5f5;
    --band: #ff6b6b;
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
    animation: breathe 2.4s ease-in-out infinite;
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

  .torso {
    fill: var(--skin);
    stroke: var(--skin-dark);
    stroke-width: 2;
  }
  .vest {
    fill: url(#flex-vest);
  }
  .head {
    fill: var(--skin);
    stroke: var(--skin-dark);
    stroke-width: 2;
  }
  .head-shine {
    fill: rgba(255, 255, 255, 0.3);
    filter: blur(2px);
  }
  .band {
    fill: var(--band);
  }

  .upper-arm,
  .forearm,
  .bicep,
  .fist {
    fill: var(--skin);
    stroke: var(--skin-dark);
    stroke-width: 2;
  }
  .arm-left {
    transform-origin: 58px 106px;
  }
  .arm-right {
    transform-origin: 142px 106px;
  }

  .shorts {
    fill: var(--shorts);
  }
  .shin {
    fill: var(--skin);
    stroke: var(--skin-dark);
    stroke-width: 2;
  }
  .shoe {
    fill: var(--shoe);
    stroke: #c4c4c4;
    stroke-width: 2;
  }
  .leg-left {
    transform-origin: 89px 152px;
  }
  .leg-right {
    transform-origin: 111px 152px;
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

  .brow {
    fill: none;
    stroke: #5a3d2b;
    stroke-width: 3;
    stroke-linecap: round;
    transition: transform 200ms ease;
  }
  .mouth {
    fill: none;
    stroke: #7a4a32;
    stroke-width: 3;
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
  .sweat {
    fill: #9fd9f6;
    opacity: 0;
  }

  .prop-shaker rect {
    fill: #e8503a;
    stroke: #b83c2a;
    stroke-width: 1.5;
  }
  .prop-shaker path {
    stroke: rgba(255, 255, 255, 0.6);
    stroke-width: 2;
  }
  .prop-towel {
    fill: #fdfbf4;
    stroke: #d6d0c4;
    stroke-width: 1.5;
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
    transform: translate(48px, 36px);
  }
  .s2 {
    transform: translate(154px, 64px);
  }

  /* --- A purposeful march: knees drive up, arms pump in opposition,
         shoulders stay square. Nothing like the animals' bob. --- */
  .state-walk_in .leg-left,
  .state-walk_out .leg-left,
  .state-walk_in .arm-right,
  .state-walk_out .arm-right {
    animation: march-a calc(400ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .leg-right,
  .state-walk_out .leg-right,
  .state-walk_in .arm-left,
  .state-walk_out .arm-left {
    animation: march-b calc(400ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes march-a {
    0%,
    100% {
      transform: rotate(-22deg);
    }
    50% {
      transform: rotate(22deg);
    }
  }
  @keyframes march-b {
    0%,
    100% {
      transform: rotate(22deg);
    }
    50% {
      transform: rotate(-22deg);
    }
  }
  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.4s ease-in-out infinite,
      march-bob calc(400ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes march-bob {
    0%,
    50%,
    100% {
      transform: translateY(0);
    }
    25%,
    75% {
      transform: translateY(-5px);
    }
  }

  /* --- States --- */
  .state-ask .prop-shaker,
  .state-ask .prop-towel,
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

  /* The payoff pose: both arms curl into a double bicep flex */
  .state-happy .arm-left {
    animation: flex-left 460ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
  .state-happy .arm-right {
    animation: flex-right 460ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
  .state-happy .bicep {
    animation: bicep-pump 460ms ease-out 2;
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
  .state-happy .sweat {
    opacity: 1;
    animation: sweat-fly 900ms ease-out infinite;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes flex-left {
    to {
      transform: rotate(118deg) translate(-6px, 14px);
    }
  }
  @keyframes flex-right {
    to {
      transform: rotate(-118deg) translate(6px, 14px);
    }
  }
  @keyframes bicep-pump {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.22);
    }
  }
  @keyframes sweat-fly {
    0% {
      opacity: 0;
      transform: translate(0, 0);
    }
    25% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: translate(12px, -14px);
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

  /* Hands on hips — the "fine, next time" coach pose */
  .state-shrug .arm-left {
    transform: rotate(62deg) translate(10px, 2px);
  }
  .state-shrug .arm-right {
    transform: rotate(-62deg) translate(-10px, 2px);
  }
  .state-shrug .brow-left {
    transform: translateY(-3px) rotate(6deg);
  }
  .state-shrug .brow-right {
    transform: translateY(-3px) rotate(-6deg);
  }

  .state-sad .head-group {
    transform: translateY(6px) rotate(-3deg);
  }
  .state-sad .arm-left,
  .state-sad .arm-right {
    transform: translateY(5px);
  }
  .state-sad .brow-left {
    transform: translateY(2px) rotate(-10deg);
  }
  .state-sad .brow-right {
    transform: translateY(2px) rotate(10deg);
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
      transform: rotate(-120deg) translate(4px, 10px);
    }
    50% {
      transform: rotate(-150deg) translate(4px, 10px);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 92px;
  }
  /* Light on the feet, like someone warming up */
  .gesture-bounce.state-idle .body-svg {
    animation:
      breathe 2.4s ease-in-out infinite,
      idle-bounce 360ms ease-in-out 4;
  }
  @keyframes idle-bounce {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-6px);
    }
  }

  .gesture-neckroll.state-idle .head-group {
    animation: neckroll 1400ms ease-in-out;
  }
  @keyframes neckroll {
    0%,
    100% {
      transform: rotate(0deg);
    }
    30% {
      transform: rotate(-12deg);
    }
    65% {
      transform: rotate(12deg);
    }
  }

  .gesture-jab.state-idle .arm-right {
    animation: jab 320ms ease-in-out 3;
  }
  @keyframes jab {
    0%,
    100% {
      transform: rotate(0deg) translateY(0);
    }
    50% {
      transform: rotate(-74deg) translate(2px, 6px);
    }
  }
</style>
