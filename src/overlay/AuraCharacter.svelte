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
    idleGestures: ["scan", "pulse", "analyse"],
    eyeHeightRatio: 0.45,
    maxPupilOffset: 7,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let idleGesture = $derived(behaviour.idleGesture);

  // Ticks around the outer ring — generated so the spacing is exact.
  const TICKS = Array.from({ length: 24 }, (_, i) => i * 15);
  // Equaliser bars that react to each state.
  const BARS = [0, 1, 2, 3, 4, 5, 6];
</script>

<div
  class="aura state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <radialGradient id="aura-core" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#ffffff" />
        <stop offset="35%" stop-color="var(--glow-bright)" />
        <stop offset="100%" stop-color="var(--glow)" />
      </radialGradient>
      <radialGradient id="aura-halo" cx="50%" cy="50%" r="50%">
        <stop offset="60%" stop-color="transparent" />
        <stop offset="100%" stop-color="var(--glow-soft)" />
      </radialGradient>
    </defs>

    <!-- A projection casts light, not shadow. -->
    <ellipse class="ground-glow" cx="100" cy="198" rx="44" ry="8" />

    <!-- Projector base: the only solid object, implying it's beamed up -->
    <g class="emitter">
      <ellipse class="emitter-top" cx="100" cy="186" rx="26" ry="7" />
      <path class="emitter-body" d="M 76 186 L 124 186 L 118 198 L 82 198 Z" />
      <ellipse class="emitter-lens" cx="100" cy="186" rx="15" ry="4" />
    </g>

    <!-- The projected beam, widening toward the hologram -->
    <path class="beam" d="M 86 184 L 114 184 L 150 92 L 50 92 Z" />

    <g class="holo-group">
      <circle class="halo" cx="100" cy="108" r="62" />

      <!-- Outer ring, ticking round like a scanner -->
      <g class="ring ring-outer">
        <circle class="ring-line" cx="100" cy="108" r="54" />
        {#each TICKS as angle (angle)}
          <line
            class="tick"
            x1="100"
            y1="48"
            x2="100"
            y2="56"
            style="--a: {angle}deg"
          />
        {/each}
      </g>

      <!-- Mid ring, counter-rotating with an open arc -->
      <g class="ring ring-mid">
        <path class="arc" d="M 58 108 A 42 42 0 0 1 142 108" />
        <path class="arc" d="M 142 108 A 42 42 0 0 1 58 108" opacity="0.35" />
      </g>

      <!-- Inner ring, following the cursor like a gaze -->
      <g
        class="ring ring-inner"
        style="transform: translate({pupilOffset.x}px, {pupilOffset.y}px)"
      >
        <circle class="ring-line bright" cx="100" cy="108" r="28" />
        <circle class="core" cx="100" cy="108" r="16" />
        <circle class="core-dot" cx="100" cy="108" r="6" />
      </g>

      <!-- Waveform bars: the closest thing it has to an expression -->
      <g class="bars">
        {#each BARS as i (i)}
          <rect class="bar" x={76 + i * 8} y="142" width="4" height="12" rx="2" style="--i: {i}" />
        {/each}
      </g>

      {#if prop === "lookfar"}
        <!-- A distant target lock, far off to one side -->
        <g class="prop-lock">
          <rect x="146" y="76" width="26" height="26" rx="3" />
          <line x1="159" y1="70" x2="159" y2="80" />
          <line x1="159" y1="98" x2="159" y2="108" />
        </g>
      {:else if prop !== "stretch" && prop !== "posture"}
        <!-- A hydration readout rather than a physical object -->
        <g class="prop-readout">
          <rect x="142" y="130" width="34" height="22" rx="4" />
          <path class="readout-wave" d="M 147 143 L 152 137 L 157 146 L 162 135 L 167 142 L 172 139" />
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
  .aura {
    --scale: 1;
    --glow: #2fa8ff;
    --glow-bright: #7fd4ff;
    --glow-soft: rgba(47, 168, 255, 0.18);
    --emitter: #39465a;
    --emitter-dark: #222b38;

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

  /* Light pooling under the emitter, instead of a cast shadow. */
  .ground-glow {
    fill: var(--glow);
    opacity: 0.22;
    filter: blur(5px);
  }

  .emitter-top {
    fill: var(--emitter);
  }
  .emitter-body {
    fill: var(--emitter-dark);
  }
  .emitter-lens {
    fill: var(--glow-bright);
    filter: drop-shadow(0 0 4px var(--glow));
  }

  .beam {
    fill: var(--glow);
    opacity: 0.1;
  }

  .halo {
    fill: url(#aura-halo);
    opacity: 0.55;
  }

  .ring-line {
    fill: none;
    stroke: var(--glow);
    stroke-width: 2;
    opacity: 0.75;
  }
  .ring-line.bright {
    stroke: var(--glow-bright);
    stroke-width: 2.5;
    opacity: 1;
  }
  .arc {
    fill: none;
    stroke: var(--glow-bright);
    stroke-width: 3;
    stroke-linecap: round;
  }
  .tick {
    stroke: var(--glow);
    stroke-width: 2;
    stroke-linecap: round;
    opacity: 0.7;
    transform: rotate(var(--a));
    transform-origin: 100px 108px;
  }

  .core {
    fill: url(#aura-core);
    opacity: 0.9;
    filter: drop-shadow(0 0 6px var(--glow));
  }
  .core-dot {
    fill: #ffffff;
  }

  /* Rings turn at different speeds and directions — the detail that
     sells "machine" rather than "decoration". */
  .ring-outer {
    transform-origin: 100px 108px;
    animation: spin-cw 22s linear infinite;
  }
  .ring-mid {
    transform-origin: 100px 108px;
    animation: spin-ccw 14s linear infinite;
  }
  .ring-inner {
    transition: transform 120ms ease-out;
  }
  @keyframes spin-cw {
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes spin-ccw {
    to {
      transform: rotate(-360deg);
    }
  }

  .holo-group {
    animation: holo-float 3.6s ease-in-out infinite;
    transform-origin: 100px 108px;
  }
  @keyframes holo-float {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-5px);
    }
  }

  .bar {
    fill: var(--glow-bright);
    transform-origin: center bottom;
    transform-box: fill-box;
    animation: bar-idle 1.4s ease-in-out infinite;
    animation-delay: calc(var(--i) * 90ms);
  }
  @keyframes bar-idle {
    0%,
    100% {
      transform: scaleY(0.4);
    }
    50% {
      transform: scaleY(1);
    }
  }

  .prop-readout rect,
  .prop-lock rect {
    fill: rgba(47, 168, 255, 0.12);
    stroke: var(--glow);
    stroke-width: 1.5;
  }
  .readout-wave {
    fill: none;
    stroke: var(--glow-bright);
    stroke-width: 2;
    stroke-linecap: round;
  }
  .prop-lock line {
    stroke: var(--glow-bright);
    stroke-width: 2;
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: var(--glow-bright);
  }
  .s1 {
    transform: translate(40px, 48px);
  }
  .s2 {
    transform: translate(162px, 74px);
  }

  /* --- Materialises rather than walks: rings spin up fast and the
         whole projection resolves into focus. --- */
  .state-walk_in .holo-group {
    animation: boot calc(900ms / var(--anim-speed, 1)) ease-out both;
  }
  .state-walk_out .holo-group {
    animation: shutdown calc(700ms / var(--anim-speed, 1)) ease-in both;
  }
  @keyframes boot {
    0% {
      opacity: 0;
      transform: scale(0.3) rotate(-90deg);
      filter: blur(6px);
    }
    60% {
      opacity: 1;
      transform: scale(1.08) rotate(8deg);
      filter: blur(0);
    }
    100% {
      opacity: 1;
      transform: scale(1) rotate(0deg);
    }
  }
  @keyframes shutdown {
    0% {
      opacity: 1;
      transform: scale(1);
    }
    100% {
      opacity: 0;
      transform: scale(0.2) rotate(60deg);
      filter: blur(5px);
    }
  }
  .state-walk_in .ring-outer,
  .state-walk_out .ring-outer {
    animation: spin-cw calc(1.6s / var(--anim-speed, 1)) linear infinite;
  }
  .state-walk_in .ring-mid,
  .state-walk_out .ring-mid {
    animation: spin-ccw calc(1.1s / var(--anim-speed, 1)) linear infinite;
  }
  .state-walk_in .beam,
  .state-walk_out .beam {
    animation: beam-flicker 180ms steps(2) infinite;
  }
  @keyframes beam-flicker {
    0% {
      opacity: 0.16;
    }
    100% {
      opacity: 0.07;
    }
  }

  /* --- States --- */
  .state-ask .prop-readout,
  .state-ask .prop-lock {
    animation: prop-pulse 1.8s ease-in-out infinite;
  }
  @keyframes prop-pulse {
    0%,
    100% {
      opacity: 0.75;
    }
    50% {
      opacity: 1;
    }
  }

  /* Confirmation: everything flares green and the bars spike */
  .state-happy {
    --glow: #35e3a0;
    --glow-bright: #9bffd8;
    --glow-soft: rgba(53, 227, 160, 0.22);
  }
  .state-happy .core {
    animation: confirm-flare 520ms ease-out;
  }
  .state-happy .ring-outer {
    animation: spin-cw 3s linear infinite;
  }
  .state-happy .bar {
    animation: bar-spike 260ms ease-in-out infinite;
    animation-delay: calc(var(--i) * 50ms);
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes confirm-flare {
    0%,
    100% {
      transform: scale(1);
      filter: drop-shadow(0 0 6px var(--glow));
    }
    45% {
      transform: scale(1.35);
      filter: drop-shadow(0 0 16px var(--glow));
    }
  }
  @keyframes bar-spike {
    0%,
    100% {
      transform: scaleY(0.5);
    }
    50% {
      transform: scaleY(2.4);
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

  /* Deferred: amber, and the waveform flattens to a holding pattern */
  .state-shrug {
    --glow: #ffb13b;
    --glow-bright: #ffd79a;
    --glow-soft: rgba(255, 177, 59, 0.2);
  }
  .state-shrug .bar {
    animation: none;
    transform: scaleY(0.45);
  }
  .state-shrug .ring-mid {
    animation: spin-ccw 30s linear infinite;
  }

  /* Ignored: signal degrades, rings slow, the image glitches */
  .state-sad {
    --glow: #6b7a8f;
    --glow-bright: #9aa8ba;
    --glow-soft: rgba(107, 122, 143, 0.14);
  }
  .state-sad .holo-group {
    animation: signal-loss 2.2s ease-in-out infinite;
  }
  .state-sad .ring-outer {
    animation: spin-cw 60s linear infinite;
  }
  .state-sad .bar {
    animation: none;
    transform: scaleY(0.2);
  }
  .state-sad .beam {
    opacity: 0.05;
  }
  @keyframes signal-loss {
    0%,
    100% {
      opacity: 0.8;
      transform: translateY(2px) skewX(0deg);
    }
    48% {
      opacity: 0.55;
      transform: translateY(2px) skewX(2deg);
    }
    52% {
      opacity: 0.9;
      transform: translateY(2px) skewX(-2deg);
    }
  }

  /* No limbs to wave, so the rings tip toward you instead */
  .state-wave .holo-group {
    animation: holo-nod 460ms ease-in-out 3;
  }
  @keyframes holo-nod {
    0%,
    100% {
      transform: rotate(0deg) scale(1);
    }
    50% {
      transform: rotate(-12deg) scale(1.05);
    }
  }

  /* --- Idle gestures --- */
  /* A scan line sweeps the rings */
  .gesture-scan.state-idle .ring-mid {
    animation: spin-ccw 1.4s linear 2;
  }
  .gesture-scan.state-idle .core {
    animation: confirm-flare 1400ms ease-in-out;
  }

  .gesture-pulse.state-idle .halo {
    animation: halo-pulse 700ms ease-in-out 2;
  }
  @keyframes halo-pulse {
    0%,
    100% {
      opacity: 0.55;
      transform: scale(1);
    }
    50% {
      opacity: 0.85;
      transform: scale(1.1);
    }
  }

  .gesture-analyse.state-idle .bar {
    animation: bar-spike 200ms ease-in-out 6;
    animation-delay: calc(var(--i) * 40ms);
  }
</style>
