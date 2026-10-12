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
    idleGestures: ["earwiggle", "nosetwitch", "thump"],
    eyeHeightRatio: 0.42,
  });

  let pupilOffset = $derived(behaviour.pupilOffset);
  let eyesClosed = $derived(behaviour.eyesClosed);
  let idleGesture = $derived(behaviour.idleGesture);
</script>

<div
  class="hop state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
  style="--scale: {scale}"
  bind:this={rootEl}
>
  <svg class="body-svg" viewBox="0 0 200 215" width="150" height="161" aria-hidden="true">
    <defs>
      <radialGradient id="hop-body" cx="40%" cy="28%" r="76%">
        <stop offset="0%" stop-color="var(--fur-light)" />
        <stop offset="62%" stop-color="var(--fur)" />
        <stop offset="100%" stop-color="var(--fur-shadow)" />
      </radialGradient>
    </defs>

    <ellipse class="ground-shadow" cx="100" cy="202" rx="42" ry="7" />

    <!-- Powerful hind feet, long and flat on the ground -->
    <g class="hind-foot hind-left">
      <ellipse cx="76" cy="194" rx="22" ry="10" />
    </g>
    <g class="hind-foot hind-right">
      <ellipse cx="124" cy="194" rx="22" ry="10" />
    </g>

    <!-- Round cotton tail -->
    <circle class="cottontail" cx="150" cy="166" r="13" />

    <g class="body-squash">
      <!-- Pear-shaped body: narrow chest, heavy haunches -->
      <path
        class="body"
        d="M 100 104
           C 122 104 136 134 138 162
           C 140 184 122 192 100 192
           C 78 192 60 184 62 162
           C 64 134 78 104 100 104 Z"
      />
      <path class="belly" d="M 100 130 C 112 130 118 154 116 172 C 114 184 86 184 84 172 C 82 154 88 130 100 130 Z" />

      <!-- Short front paws held up at the chest -->
      <g class="paw-arm paw-left">
        <ellipse cx="74" cy="146" rx="10" ry="13" />
      </g>
      <g class="paw-arm paw-right">
        <ellipse cx="126" cy="146" rx="10" ry="13" />
      </g>
    </g>

    <g class="head-group">
      <!-- The ears: tall, upright, and the whole silhouette cue -->
      <g class="ear ear-left">
        <ellipse class="ear-outer" cx="83" cy="40" rx="13" ry="40" />
        <ellipse class="ear-inner" cx="83" cy="42" rx="7" ry="32" />
      </g>
      <g class="ear ear-right">
        <ellipse class="ear-outer" cx="117" cy="40" rx="13" ry="40" />
        <ellipse class="ear-inner" cx="117" cy="42" rx="7" ry="32" />
      </g>

      <ellipse class="head" cx="100" cy="104" rx="40" ry="36" />
      <ellipse class="head-shine" cx="82" cy="88" rx="14" ry="10" />

      <ellipse class="blush" cx="70" cy="114" rx="9" ry="5" />
      <ellipse class="blush" cx="130" cy="114" rx="9" ry="5" />

      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-white" cx="84" cy="100" r="11" />
        <circle class="pupil" cx={84 + pupilOffset.x} cy={100 + pupilOffset.y} r="5.5" />
        <circle class="eye-shine" cx={84 + pupilOffset.x - 2} cy={100 + pupilOffset.y - 2.5} r="2" />
        <path class="eyelid" d="M 73 100 Q 84 100 95 100" />
      </g>
      <g class="eye" class:closed={eyesClosed}>
        <circle class="eye-white" cx="116" cy="100" r="11" />
        <circle class="pupil" cx={116 + pupilOffset.x} cy={100 + pupilOffset.y} r="5.5" />
        <circle class="eye-shine" cx={116 + pupilOffset.x - 2} cy={100 + pupilOffset.y - 2.5} r="2" />
        <path class="eyelid" d="M 105 100 Q 116 100 127 100" />
      </g>

      <!-- Nose, split lip and buck teeth -->
      <path class="nose" d="M 94 116 L 106 116 L 100 123 Z" />
      <path class="lip" d="M 100 123 L 100 129 M 100 129 Q 92 134 88 128 M 100 129 Q 108 134 112 128" />
      <g class="teeth">
        <rect x="93" y="130" width="6" height="9" rx="1.5" />
        <rect x="101" y="130" width="6" height="9" rx="1.5" />
      </g>

      <g class="whiskers">
        <path d="M 74 122 L 48 118" />
        <path d="M 74 128 L 50 132" />
        <path d="M 126 122 L 152 118" />
        <path d="M 126 128 L 150 132" />
      </g>

      <ellipse class="tear" cx="88" cy="112" rx="3" ry="5" />

      {#if prop === "lookfar"}
        <ellipse class="prop-lookfar" cx="128" cy="86" rx="10" ry="7" />
      {:else if prop !== "stretch" && prop !== "posture"}
        <!-- A carrot, of course -->
        <g class="prop-carrot">
          <path class="carrot-body" d="M 130 142 L 142 136 L 146 170 Z" />
          <path class="carrot-top" d="M 132 141 Q 124 130 132 126 M 137 138 Q 134 125 142 123" />
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
  .hop {
    --scale: 1;
    --fur-light: #fdf6ee;
    --fur: #ead9c6;
    --fur-shadow: #c9b199;
    --fur-dark: #8c7360;
    --belly: #fffdf8;
    --inner-ear: #f7adc0;
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
      transform: scaleX(1.03) scaleY(0.97);
    }
  }

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.16);
    filter: blur(1.5px);
  }

  .body {
    fill: url(#hop-body);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .belly {
    fill: var(--belly);
    opacity: 0.9;
  }
  .head {
    fill: url(#hop-body);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .head-shine {
    fill: rgba(255, 255, 255, 0.45);
    filter: blur(2px);
  }

  .hind-foot ellipse {
    fill: var(--fur);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .hind-left {
    transform-origin: 76px 194px;
  }
  .hind-right {
    transform-origin: 124px 194px;
  }

  .paw-arm ellipse {
    fill: var(--fur-light);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .paw-left {
    transform-origin: 74px 136px;
  }
  .paw-right {
    transform-origin: 126px 136px;
  }

  .cottontail {
    fill: var(--belly);
    stroke: var(--fur-shadow);
    stroke-width: 2;
  }

  .ear-outer {
    fill: var(--fur);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .ear-inner {
    fill: var(--inner-ear);
  }
  .ear-left {
    transform-origin: 83px 78px;
  }
  .ear-right {
    transform-origin: 117px 78px;
  }

  .eye-white {
    fill: white;
    stroke: var(--fur-dark);
    stroke-width: 1.5;
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
  .lip {
    fill: none;
    stroke: var(--fur-dark);
    stroke-width: 2;
    stroke-linecap: round;
  }
  .teeth rect {
    fill: #fffdf6;
    stroke: var(--fur-dark);
    stroke-width: 1.2;
  }
  .whiskers {
    fill: none;
    stroke: rgba(90, 70, 55, 0.4);
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

  .carrot-body {
    fill: #f08b3a;
    stroke: #c26a24;
    stroke-width: 1.5;
  }
  .carrot-top {
    fill: none;
    stroke: #4fa85c;
    stroke-width: 3;
    stroke-linecap: round;
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
    transform: translate(44px, 50px);
  }
  .s2 {
    transform: translate(158px, 76px);
  }

  /* --- Big arcing leaps. The body rises and falls in a tall parabola,
         ears trail behind on the way up and whip forward on landing. --- */
  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.6s ease-in-out infinite,
      leap calc(620ms / var(--anim-speed, 1)) cubic-bezier(0.3, 0, 0.6, 1) infinite;
    transform-origin: 100px 200px;
  }
  @keyframes leap {
    0% {
      transform: translateY(0) scale(1.08, 0.9);
    }
    20% {
      transform: translateY(-34px) scale(0.94, 1.1);
    }
    45% {
      transform: translateY(-46px) scale(1, 1);
    }
    75% {
      transform: translateY(-10px) scale(1.02, 0.98);
    }
    88% {
      transform: translateY(0) scale(1.12, 0.86);
    }
    100% {
      transform: translateY(0) scale(1.08, 0.9);
    }
  }
  .state-walk_in .ear,
  .state-walk_out .ear {
    animation: ear-trail calc(620ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes ear-trail {
    0%,
    100% {
      transform: rotate(0deg);
    }
    25% {
      transform: rotate(26deg);
    }
    70% {
      transform: rotate(-16deg);
    }
  }
  .state-walk_in .hind-foot,
  .state-walk_out .hind-foot {
    animation: foot-tuck calc(620ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes foot-tuck {
    0%,
    88%,
    100% {
      transform: scaleY(1) translateY(0);
    }
    35% {
      transform: scaleY(0.7) translateY(-6px);
    }
  }

  /* --- States --- */
  .state-ask .prop-carrot,
  .state-ask .prop-lookfar {
    animation: prop-bob 1.6s ease-in-out infinite;
  }
  @keyframes prop-bob {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    50% {
      transform: translateY(-3px) rotate(-6deg);
    }
  }

  /* Ears spring bolt upright and the whole body double-hops */
  .state-happy .eye .eyelid {
    opacity: 1;
    transform: translateY(-3px) scaleY(1.3);
  }
  .state-happy .eye-white,
  .state-happy .pupil,
  .state-happy .eye-shine {
    display: none;
  }
  .state-happy .ear {
    animation: ear-spring 420ms cubic-bezier(0.34, 1.56, 0.64, 1) infinite;
  }
  .state-happy .body-squash {
    animation: double-hop 420ms ease-in-out infinite;
    transform-origin: 100px 192px;
  }
  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes ear-spring {
    0%,
    100% {
      transform: rotate(0deg) scaleY(1);
    }
    40% {
      transform: rotate(0deg) scaleY(1.12);
    }
  }
  @keyframes double-hop {
    0%,
    100% {
      transform: translateY(0) scale(1.06, 0.94);
    }
    30% {
      transform: translateY(-14px) scale(0.96, 1.06);
    }
    60% {
      transform: translateY(-4px) scale(1, 1);
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

  /* Ears fold flat back — the rabbit version of a shrug */
  .state-shrug .ear-left {
    transform: rotate(-54deg) translateY(10px);
  }
  .state-shrug .ear-right {
    transform: rotate(54deg) translateY(10px);
  }
  .state-shrug .paw-arm {
    transform: translateY(-6px);
  }

  .state-sad .ear-left {
    transform: rotate(-34deg) translateY(16px);
  }
  .state-sad .ear-right {
    transform: rotate(34deg) translateY(16px);
  }
  .state-sad .head-group {
    transform: translateY(6px) rotate(-3deg);
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

  .state-wave .paw-right {
    animation: wave 500ms ease-in-out 3;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg) translateY(0);
    }
    50% {
      transform: rotate(-38deg) translateY(-22px);
    }
  }

  /* --- Idle gestures --- */
  .head-group {
    transform-origin: 100px 140px;
  }
  .gesture-earwiggle.state-idle .ear-left {
    animation: wiggle-a 420ms ease-in-out 3;
  }
  .gesture-earwiggle.state-idle .ear-right {
    animation: wiggle-b 420ms ease-in-out 3;
  }
  @keyframes wiggle-a {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-13deg);
    }
  }
  @keyframes wiggle-b {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(13deg);
    }
  }

  .gesture-nosetwitch.state-idle .nose,
  .gesture-nosetwitch.state-idle .whiskers {
    animation: twitch 180ms ease-in-out 5;
    transform-origin: 100px 120px;
  }
  @keyframes twitch {
    0%,
    100% {
      transform: translateY(0) scaleX(1);
    }
    50% {
      transform: translateY(-1.5px) scaleX(1.08);
    }
  }

  /* The classic rabbit foot-thump */
  .gesture-thump.state-idle .hind-right {
    animation: thump 260ms ease-in-out 4;
  }
  @keyframes thump {
    0%,
    100% {
      transform: translateY(0) scaleY(1);
    }
    40% {
      transform: translateY(-8px) scaleY(0.85);
    }
    70% {
      transform: translateY(0) scaleY(1.15);
    }
  }
</style>
