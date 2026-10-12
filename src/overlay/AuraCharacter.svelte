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
    idleGestures: ["powercheck", "headturn", "repulsor"],
    eyeHeightRatio: 0.3,
    maxPupilOffset: 2,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let idleGesture = $derived(behaviour.idleGesture);
</script>

<div
  class="aura state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <linearGradient id="aura-red" x1="20%" y1="0%" x2="80%" y2="100%">
        <stop offset="0%" stop-color="var(--red-light)" />
        <stop offset="55%" stop-color="var(--red)" />
        <stop offset="100%" stop-color="var(--red-dark)" />
      </linearGradient>
      <linearGradient id="aura-gold" x1="20%" y1="0%" x2="80%" y2="100%">
        <stop offset="0%" stop-color="var(--gold-light)" />
        <stop offset="55%" stop-color="var(--gold)" />
        <stop offset="100%" stop-color="var(--gold-dark)" />
      </linearGradient>
      <radialGradient id="aura-reactor" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#ffffff" />
        <stop offset="40%" stop-color="var(--arc-bright)" />
        <stop offset="100%" stop-color="var(--arc)" />
      </radialGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="202" rx="42" ry="7" />

    <!-- Boot thrusters, flaring during flight -->
    <g class="thruster thruster-left">
      <path class="flame" d="M 80 196 Q 74 212 80 220 Q 86 212 80 196 Z" />
    </g>
    <g class="thruster thruster-right">
      <path class="flame" d="M 120 196 Q 114 212 120 220 Q 126 212 120 196 Z" />
    </g>

    <!-- Legs: gold shin plating over red thighs -->
    <g class="leg leg-left">
      <path class="plate-red" d="M 84 152 L 98 152 L 96 176 L 84 176 Z" />
      <path class="plate-gold" d="M 84 176 L 96 176 L 95 194 L 83 194 Z" />
      <path class="boot" d="M 80 192 L 98 192 L 100 200 L 78 200 Z" />
    </g>
    <g class="leg leg-right">
      <path class="plate-red" d="M 102 152 L 116 152 L 116 176 L 104 176 Z" />
      <path class="plate-gold" d="M 104 176 L 116 176 L 117 194 L 105 194 Z" />
      <path class="boot" d="M 102 192 L 120 192 L 122 200 L 100 200 Z" />
    </g>

    <g class="torso-group">
      <!-- Armoured chest: wide pauldrons tapering to the waist -->
      <path
        class="plate-red chest"
        d="M 100 86
           L 134 94
           Q 142 100 140 118
           L 134 146
           Q 100 156 66 146
           L 60 118
           Q 58 100 66 94 Z"
      />
      <!-- Gold ab plating, segmented -->
      <path class="plate-gold" d="M 76 136 L 124 136 L 120 150 Q 100 156 80 150 Z" />
      <path class="seam" d="M 78 142 L 122 142" />

      <!-- The arc reactor: the single most recognisable detail -->
      <g class="reactor">
        <circle class="reactor-housing" cx="100" cy="112" r="17" />
        <circle class="reactor-ring" cx="100" cy="112" r="12" />
        <circle class="reactor-core" cx="100" cy="112" r="8" />
        <circle class="reactor-hot" cx="100" cy="112" r="4" />
      </g>

      <!-- Shoulder pauldrons, overlapping the arms -->
      <path class="plate-gold pauldron" d="M 56 92 Q 46 98 48 114 L 66 118 Q 70 100 66 92 Z" />
      <path class="plate-gold pauldron" d="M 144 92 Q 154 98 152 114 L 134 118 Q 130 100 134 92 Z" />

      <!-- Arms with repulsor palms -->
      <g class="arm arm-left">
        <path class="plate-red" d="M 46 114 L 62 118 L 58 148 L 44 144 Z" />
        <path class="plate-gold" d="M 44 144 L 58 148 L 56 164 L 42 160 Z" />
        <circle class="repulsor" cx="49" cy="166" r="8" />
      </g>
      <g class="arm arm-right">
        <path class="plate-red" d="M 154 114 L 138 118 L 142 148 L 156 144 Z" />
        <path class="plate-gold" d="M 156 144 L 142 148 L 144 164 L 158 160 Z" />
        <circle class="repulsor" cx="151" cy="166" r="8" />
      </g>
    </g>

    <g class="head-group">
      <!-- Helmet: red shell with the gold faceplate inset -->
      <path
        class="plate-red helmet"
        d="M 100 30
           Q 128 30 132 54
           L 132 72
           Q 126 86 100 88
           Q 74 86 68 72
           L 68 54
           Q 72 30 100 30 Z"
      />
      <path
        class="plate-gold faceplate"
        d="M 100 42
           Q 120 42 123 58
           L 122 72
           Q 116 82 100 83
           Q 84 82 78 72
           L 77 58
           Q 80 42 100 42 Z"
      />
      <!-- Cheek vents and jaw seam -->
      <path class="seam" d="M 82 70 L 90 70 M 110 70 L 118 70" />
      <path class="seam" d="M 86 78 Q 100 84 114 78" />

      <!-- The glowing slit eyes, angled inward -->
      <g class="eyes" style="transform: translate({pupilOffset.x}px, {pupilOffset.y}px)">
        <path class="eye-slit" d="M 82 58 L 95 55 L 95 63 L 82 65 Z" />
        <path class="eye-slit" d="M 118 58 L 105 55 L 105 63 L 118 65 Z" />
      </g>
    </g>

    {#if prop === "lookfar"}
      <!-- HUD target lock projected out to the side -->
      <g class="prop-hud">
        <rect x="144" y="54" width="30" height="30" rx="2" />
        <line x1="159" y1="48" x2="159" y2="58" />
        <line x1="159" y1="80" x2="159" y2="90" />
        <circle cx="159" cy="69" r="5" />
      </g>
    {:else if prop !== "stretch" && prop !== "posture"}
      <!-- Status readout panel, in the suit's own HUD style -->
      <g class="prop-hud">
        <rect x="142" y="120" width="36" height="24" rx="2" />
        <path class="hud-wave" d="M 147 134 L 152 127 L 157 138 L 162 125 L 167 133 L 173 130" />
      </g>
    {/if}

    <g class="sparkles">
      <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
    </g>
  </svg>
</div>

<style>
  .aura {
    --scale: 1;
    /* Hot red and polished gold — the palette is most of the read. */
    --red-light: #e8453c;
    --red: #b4201c;
    --red-dark: #76100f;
    --gold-light: #ffe08a;
    --gold: #e0ac28;
    --gold-dark: #a77512;
    --arc: #5fd8ff;
    --arc-bright: #d6f6ff;
    --seam: rgba(0, 0, 0, 0.35);

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
    fill: rgba(0, 0, 0, 0.18);
    filter: blur(1.5px);
  }

  .plate-red {
    fill: url(#aura-red);
    stroke: var(--red-dark);
    stroke-width: 1.6;
    stroke-linejoin: round;
  }
  .plate-gold {
    fill: url(#aura-gold);
    stroke: var(--gold-dark);
    stroke-width: 1.6;
    stroke-linejoin: round;
  }
  .boot {
    fill: url(#aura-gold);
    stroke: var(--gold-dark);
    stroke-width: 1.6;
  }
  .seam {
    fill: none;
    stroke: var(--seam);
    stroke-width: 1.6;
    stroke-linecap: round;
  }

  /* The arc reactor is always lit and always pulsing — it reads as the
     power source even when nothing else is moving. */
  .reactor-housing {
    fill: #2a2a30;
    stroke: var(--gold-dark);
    stroke-width: 2;
  }
  .reactor-ring {
    fill: none;
    stroke: var(--arc);
    stroke-width: 2.5;
    opacity: 0.85;
  }
  .reactor-core {
    fill: url(#aura-reactor);
    filter: drop-shadow(0 0 6px var(--arc));
  }
  .reactor-hot {
    fill: #ffffff;
  }
  .reactor {
    animation: reactor-pulse 2.6s ease-in-out infinite;
    transform-origin: 100px 112px;
  }
  @keyframes reactor-pulse {
    0%,
    100% {
      opacity: 0.9;
      transform: scale(1);
    }
    50% {
      opacity: 1;
      transform: scale(1.06);
    }
  }

  .eye-slit {
    fill: var(--arc-bright);
    filter: drop-shadow(0 0 5px var(--arc));
    transition: transform 120ms ease-out;
  }
  .eyes {
    transition: transform 120ms ease-out;
  }

  .repulsor {
    fill: var(--arc-bright);
    opacity: 0.55;
    filter: drop-shadow(0 0 3px var(--arc));
  }

  .arm-left {
    transform-origin: 54px 116px;
  }
  .arm-right {
    transform-origin: 146px 116px;
  }
  .leg-left {
    transform-origin: 91px 152px;
  }
  .leg-right {
    transform-origin: 109px 152px;
  }

  /* Thruster flames stay hidden until flight. */
  .flame {
    fill: var(--arc-bright);
    opacity: 0;
    filter: drop-shadow(0 0 6px var(--arc));
    transform-origin: top center;
  }

  .prop-hud rect {
    fill: rgba(95, 216, 255, 0.1);
    stroke: var(--arc);
    stroke-width: 1.5;
  }
  .prop-hud line,
  .prop-hud circle {
    fill: none;
    stroke: var(--arc-bright);
    stroke-width: 1.8;
  }
  .hud-wave {
    fill: none;
    stroke: var(--arc-bright);
    stroke-width: 2;
    stroke-linecap: round;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: var(--gold-light);
  }
  .s1 {
    transform: translate(44px, 32px);
  }
  .s2 {
    transform: translate(158px, 58px);
  }

  /* --- Flies in on its thrusters rather than walking: legs together,
         a forward lean, and the flames lit the whole way. --- */
  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation: fly calc(620ms / var(--anim-speed, 1)) ease-in-out infinite;
    transform-origin: 100px 200px;
  }
  @keyframes fly {
    0%,
    100% {
      transform: translateY(0) rotate(-3deg);
    }
    50% {
      transform: translateY(-11px) rotate(3deg);
    }
  }
  .state-walk_in .flame,
  .state-walk_out .flame {
    opacity: 1;
    animation: flame-flicker 120ms ease-in-out infinite alternate;
  }
  @keyframes flame-flicker {
    0% {
      transform: scaleY(0.8) scaleX(0.9);
      opacity: 0.8;
    }
    100% {
      transform: scaleY(1.3) scaleX(1.05);
      opacity: 1;
    }
  }
  .state-walk_in .leg-left,
  .state-walk_out .leg-left {
    transform: rotate(4deg);
  }
  .state-walk_in .leg-right,
  .state-walk_out .leg-right {
    transform: rotate(-4deg);
  }
  .state-walk_in .arm-left,
  .state-walk_out .arm-left {
    transform: rotate(12deg);
  }
  .state-walk_in .arm-right,
  .state-walk_out .arm-right {
    transform: rotate(-12deg);
  }

  /* --- States --- */
  .state-ask .prop-hud {
    animation: hud-pulse 1.8s ease-in-out infinite;
  }
  @keyframes hud-pulse {
    0%,
    100% {
      opacity: 0.75;
    }
    50% {
      opacity: 1;
    }
  }

  /* Confirm: repulsors fire, reactor flares, eyes brighten */
  .state-happy .repulsor {
    animation: repulsor-fire 420ms ease-out infinite;
  }
  .state-happy .arm-left {
    animation: repulsor-pose-left 460ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
  .state-happy .arm-right {
    animation: repulsor-pose-right 460ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }
  .state-happy .reactor {
    animation: reactor-flare 520ms ease-out;
  }
  .state-happy .eye-slit {
    filter: drop-shadow(0 0 10px var(--arc));
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes repulsor-fire {
    0%,
    100% {
      opacity: 0.5;
      r: 8;
    }
    50% {
      opacity: 1;
      r: 11;
    }
  }
  @keyframes repulsor-pose-left {
    to {
      transform: rotate(38deg) translate(-4px, 6px);
    }
  }
  @keyframes repulsor-pose-right {
    to {
      transform: rotate(-38deg) translate(4px, 6px);
    }
  }
  @keyframes reactor-flare {
    0%,
    100% {
      transform: scale(1);
      filter: brightness(1);
    }
    45% {
      transform: scale(1.3);
      filter: brightness(1.5);
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

  /* Deferred: a small shrug of the pauldrons, power eased back */
  .state-shrug .arm-left {
    transform: rotate(-16deg) translateY(-5px);
  }
  .state-shrug .arm-right {
    transform: rotate(16deg) translateY(-5px);
  }
  .state-shrug .reactor {
    animation-duration: 4s;
    opacity: 0.8;
  }

  /* Ignored: power drops — the reactor dims, eyes gutter, head droops */
  .state-sad .head-group {
    transform: translateY(6px) rotate(-4deg);
  }
  .state-sad .reactor {
    animation: reactor-brownout 1.8s ease-in-out infinite;
  }
  .state-sad .eye-slit {
    opacity: 0.45;
    filter: none;
    animation: eye-gutter 2.4s ease-in-out infinite;
  }
  .state-sad .repulsor {
    opacity: 0.2;
  }
  .state-sad .arm-left,
  .state-sad .arm-right {
    transform: translateY(5px);
  }
  @keyframes reactor-brownout {
    0%,
    100% {
      opacity: 0.45;
    }
    50% {
      opacity: 0.7;
    }
  }
  @keyframes eye-gutter {
    0%,
    100% {
      opacity: 0.45;
    }
    47% {
      opacity: 0.2;
    }
    53% {
      opacity: 0.6;
    }
  }

  .state-wave .arm-right {
    animation: wave 520ms ease-in-out 3;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(-104deg) translate(6px, 12px);
    }
    50% {
      transform: rotate(-132deg) translate(6px, 12px);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 88px;
  }
  /* Systems check: reactor and eyes surge together */
  .gesture-powercheck.state-idle .reactor {
    animation: reactor-flare 900ms ease-in-out;
  }
  .gesture-powercheck.state-idle .eye-slit {
    animation: eye-surge 900ms ease-in-out;
  }
  @keyframes eye-surge {
    0%,
    100% {
      filter: drop-shadow(0 0 5px var(--arc));
    }
    50% {
      filter: drop-shadow(0 0 14px var(--arc));
    }
  }

  .gesture-headturn.state-idle .head-group {
    animation: headturn 1500ms ease-in-out;
  }
  @keyframes headturn {
    0%,
    100% {
      transform: rotate(0deg);
    }
    30% {
      transform: rotate(-10deg);
    }
    70% {
      transform: rotate(10deg);
    }
  }

  /* Charges one palm, then lets it fade */
  .gesture-repulsor.state-idle .arm-right {
    animation: charge-arm 1500ms ease-in-out;
  }
  .gesture-repulsor.state-idle .repulsor {
    animation: repulsor-fire 500ms ease-out 3;
  }
  @keyframes charge-arm {
    0%,
    100% {
      transform: rotate(0deg);
    }
    35%,
    70% {
      transform: rotate(-54deg) translate(4px, 8px);
    }
  }
</style>
