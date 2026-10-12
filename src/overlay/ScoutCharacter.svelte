<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import type { SceneState } from "./director";

  interface CursorPositionPayload {
    x: number;
    y: number;
  }

  interface Props {
    sceneState: SceneState;
    /** Reminder-specific pose override for the "ask" state, e.g.
     * "bottle" | "stretch" | "lookfar" | "posture". null/undefined = the
     * character's generic ask pose (ball prop). */
    prop?: string | null;
    scale?: number;
    mirrored?: boolean;
  }

  let { sceneState, prop = null, scale = 1, mirrored = false }: Props = $props();

  let rootEl: HTMLDivElement | undefined = $state();
  let pupilOffset = $state({ x: 0, y: 0 });

  function onCursorPosition(pos: CursorPositionPayload) {
    if (!rootEl) return;
    const rect = rootEl.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height * 0.3;
    const dx = (pos.x - cx) * (mirrored ? -1 : 1);
    const dy = pos.y - cy;
    const dist = Math.hypot(dx, dy) || 1;
    const maxOffset = 3;
    pupilOffset = {
      x: (dx / dist) * Math.min(maxOffset, dist / 40),
      y: (dy / dist) * Math.min(maxOffset, dist / 40),
    };
  }

  let blinking = $state(false);
  let blinkTimer: ReturnType<typeof setTimeout> | undefined;

  function scheduleBlink() {
    const delay = 2200 + Math.random() * 3800;
    blinkTimer = setTimeout(async () => {
      blinking = true;
      await new Promise((r) => setTimeout(r, 120));
      blinking = false;
      scheduleBlink();
    }, delay);
  }

  // Procedural idle variety: every few seconds, while idle, play one small
  // random gesture (head-tilt / sniff-the-ground / tail-chase) so standing
  // still never reads as a static looping GIF. Only one plays at a time and
  // it fully resets before the next is picked.
  type IdleGesture = "none" | "tilt" | "sniff" | "tail-chase";
  let idleGesture = $state<IdleGesture>("none");
  let idleTimer: ReturnType<typeof setTimeout> | undefined;

  function scheduleIdleGesture() {
    const delay = 3200 + Math.random() * 4600;
    idleTimer = setTimeout(() => {
      if (sceneState === "idle") {
        const options: IdleGesture[] = ["tilt", "sniff", "tail-chase"];
        idleGesture = options[Math.floor(Math.random() * options.length)];
        const duration = idleGesture === "tail-chase" ? 1400 : 1000;
        setTimeout(() => {
          idleGesture = "none";
        }, duration);
      }
      scheduleIdleGesture();
    }, delay);
  }

  let unlistenCursor: (() => void) | undefined;

  onMount(() => {
    const webview = getCurrentWebviewWindow();
    webview
      .listen<CursorPositionPayload>("overlay://cursor", (event) =>
        onCursorPosition(event.payload),
      )
      .then((unlisten) => {
        unlistenCursor = unlisten;
      });
    scheduleBlink();
    scheduleIdleGesture();
  });

  onDestroy(() => {
    unlistenCursor?.();
    if (blinkTimer) clearTimeout(blinkTimer);
    if (idleTimer) clearTimeout(idleTimer);
  });

  let eyesClosed = $derived(blinking || sceneState === "sad");

  const MOUTH_PATHS: Partial<Record<SceneState, string>> = {
    happy: "M 76 148 Q 100 174 124 148",
    shrug: "M 86 152 Q 100 150 114 152",
    sad: "M 84 154 Q 100 147 116 154",
  };
  let mouthPath = $derived(MOUTH_PATHS[sceneState] ?? "M 84 148 Q 100 157 116 148");

  let isWalking = $derived(sceneState === "walk_in" || sceneState === "walk_out");
</script>

<div
  class="scout state-{sceneState} gesture-{idleGesture} prop-{prop ?? 'none'}"
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
      <radialGradient id="scout-body-shade" cx="38%" cy="30%" r="75%">
        <stop offset="0%" stop-color="var(--fur-light)" />
        <stop offset="65%" stop-color="var(--fur)" />
        <stop offset="100%" stop-color="var(--fur-shadow)" />
      </radialGradient>
      <radialGradient id="scout-head-shade" cx="36%" cy="28%" r="78%">
        <stop offset="0%" stop-color="var(--fur-light)" />
        <stop offset="60%" stop-color="var(--fur)" />
        <stop offset="100%" stop-color="var(--fur-shadow)" />
      </radialGradient>
      <linearGradient id="scout-leg-shade" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" stop-color="var(--fur)" />
        <stop offset="100%" stop-color="var(--fur-shadow)" />
      </linearGradient>
      <radialGradient id="scout-snout-shade" cx="42%" cy="25%" r="80%">
        <stop offset="0%" stop-color="#fff6e8" />
        <stop offset="100%" stop-color="var(--snout)" />
      </radialGradient>
    </defs>

    <!-- Fur-tuft texture marks: short stroke pairs along the silhouette
         edges, a classic 2D-animation shorthand for "furry" that reads at
         a glance without drawing individual hairs or costing any extra
         render weight (plain strokes, no filters). -->
    <g class="fur-tufts">
      <path d="M 54 142 l -6 4 M 58 155 l -7 2 M 56 168 l -6 1" />
      <path d="M 146 142 l 6 4 M 142 155 l 7 2 M 144 168 l 6 1" />
    </g>

    <!-- Soft contact shadow grounding the character -->
    <ellipse class="ground-shadow" cx="100" cy="206" rx="54" ry="9" />

    <!-- Tail (upright, alert, wags with follow-through) -->
    <g class="tail-root">
      <path class="tail" d="M 155 140 Q 178 110 168 78" />
    </g>

    <!-- Back legs (behind body, swing opposite to front legs). Each leg is
         a two-segment joint: the outer <g> rotates at the hip, the nested
         <g class="lower-leg"> rotates at the knee relative to the upper
         segment's own end — a rigid rotating bar reads as a stick, a
         hinged pair reads as an actual leg. -->
    <g class="leg leg-back-left" style="transform-origin: 78px 160px">
      <rect class="upper-leg" x="69" y="160" width="17" height="22" rx="8.5" />
      <g class="lower-leg" style="transform-origin: 77px 180px">
        <rect x="68" y="180" width="16" height="20" rx="8" />
        <ellipse class="foot" cx="76" cy="200" rx="13" ry="8" />
      </g>
    </g>
    <g class="leg leg-back-right" style="transform-origin: 122px 160px">
      <rect class="upper-leg" x="114" y="160" width="17" height="22" rx="8.5" />
      <g class="lower-leg" style="transform-origin: 123px 180px">
        <rect x="115" y="180" width="16" height="20" rx="8" />
        <ellipse class="foot" cx="123" cy="200" rx="13" ry="8" />
      </g>
    </g>

    <!-- Everything above the hips, pivoting as one unit at the hip point
         for the happy "stand up on hind legs" reaction — back legs stay
         planted as the support, this group rocks upright on top of them. -->
    <g class="upper-body-group">
      <!-- Body -->
      <g class="body-squash">
        <ellipse class="body" cx="100" cy="148" rx="50" ry="42" />
        <!-- Two-tone saddle: a darker patch across the back/shoulders is
             how almost every real dog's coat actually reads (darker
             back, lighter belly) — a single flat gradient alone still
             looks like one even coat, this breaks it into "fur" rather
             than "paint". -->
        <path
          class="saddle"
          d="M 58 118 Q 100 100 142 118 Q 148 140 140 158 Q 100 146 60 158 Q 52 140 58 118 Z"
        />
        <ellipse class="body-highlight" cx="82" cy="128" rx="20" ry="14" />
      </g>

      <!-- Front legs: on the ground normally, lift and tuck toward the
           chest like paws when standing up happy -->
      <g class="leg leg-front-left" style="transform-origin: 70px 150px">
        <rect class="upper-leg" x="61" y="150" width="16" height="22" rx="8" />
        <g class="lower-leg" style="transform-origin: 68px 170px">
          <rect x="60" y="170" width="15" height="20" rx="7.5" />
          <ellipse class="foot" cx="67" cy="194" rx="12" ry="7.5" />
        </g>
      </g>
      <g class="leg leg-front-right" style="transform-origin: 130px 150px">
        <rect class="upper-leg" x="123" y="150" width="16" height="22" rx="8" />
        <g class="lower-leg" style="transform-origin: 130px 170px">
          <rect x="123" y="170" width="15" height="20" rx="7.5" />
          <ellipse class="foot" cx="130" cy="194" rx="12" ry="7.5" />
        </g>
      </g>

      <!-- Chest patch -->
      <ellipse class="patch" cx="100" cy="150" rx="20" ry="18" />

      <!-- Head group: bobs/nods independently of the body during walk,
           plays idle-gesture transforms when standing still -->
      <g class="head-group">
        <!-- Ears (pointy, upright, with follow-through jiggle) -->
        <g class="ear ear-left">
          <path class="ear-outer" d="M 62 85 L 40 40 L 72 68 Z" />
          <path class="ear-inner" d="M 60 78 L 48 52 L 68 66 Z" />
        </g>
        <g class="ear ear-right">
          <path class="ear-outer" d="M 138 85 L 160 40 L 128 68 Z" />
          <path class="ear-inner" d="M 140 78 L 152 52 L 132 66 Z" />
        </g>

        <!-- Head -->
        <circle class="head" cx="100" cy="102" r="46" />
        <ellipse class="head-highlight" cx="80" cy="84" rx="16" ry="11" />
        <!-- Snout -->
        <ellipse class="snout" cx="100" cy="124" rx="24" ry="16" />

        <!-- Blush -->
        <ellipse class="blush blush-left" cx="70" cy="114" rx="8" ry="5.5" />
        <ellipse class="blush blush-right" cx="130" cy="114" rx="8" ry="5.5" />

        <!-- Eyes -->
        <g class="eye eye-left" class:closed={eyesClosed}>
          <circle class="eye-white" cx="80" cy="96" r="10.5" />
          <circle class="pupil" cx={80 + pupilOffset.x} cy={96 + pupilOffset.y} r="5" />
          <circle class="eye-shine" cx={80 + pupilOffset.x - 1.5} cy={96 + pupilOffset.y - 2} r="1.5" />
          <path class="eyelid" d="M 69 96 Q 80 96 91 96" />
        </g>
        <g class="eye eye-right" class:closed={eyesClosed}>
          <circle class="eye-white" cx="120" cy="96" r="10.5" />
          <circle class="pupil" cx={120 + pupilOffset.x} cy={96 + pupilOffset.y} r="5" />
          <circle class="eye-shine" cx={120 + pupilOffset.x - 1.5} cy={96 + pupilOffset.y - 2} r="1.5" />
          <path class="eyelid" d="M 109 96 Q 120 96 131 96" />
        </g>

        <!-- Eyebrows -->
        <path class="brow brow-left" d="M 70 80 Q 80 75 90 80" />
        <path class="brow brow-right" d="M 110 80 Q 120 75 130 80" />

        <!-- Nose -->
        <ellipse class="nose" cx="100" cy="118" rx="7" ry="5" />
        <ellipse class="nose-shine" cx="97.5" cy="116" rx="1.8" ry="1.2" />

        <!-- Mouth -->
        <path class="mouth" d={mouthPath} />

        <!-- Tongue: normal small flick for a regular smile, bigger
             panting flap for the happy stand-up -->
        <path class="tongue" d="M 90 147 Q 100 172 110 147 Q 100 158 90 147 Z" />

        <!-- Tear for sad -->
        <ellipse class="tear" cx="90" cy="104" rx="3" ry="5" />

        <!-- Reminder-specific "ask" prop: a ball for the generic/water
             case, or a pose matched to what the reminder is actually
             asking for (look into the distance, stretch, sit up
             straight) — one shared vocabulary any character can plug
             poses into via the `prop` id from the reminder config. -->
        {#if prop === "lookfar"}
          <!-- A paw raised to shield/peer, as if looking into the
               distance — echoes someone shielding their eyes from the
               sun to focus on something far away. -->
          <g class="prop-lookfar">
            <ellipse cx="124" cy="88" rx="10" ry="7" />
          </g>
        {:else if prop !== "stretch" && prop !== "posture"}
          <g class="prop-ball">
            <circle cx="134" cy="128" r="11" />
            <path d="M 126 122 Q 134 128 126 134" />
            <path d="M 142 122 Q 134 128 142 134" />
          </g>
        {/if}
      </g>

      <!-- Sparkles for happy -->
      <g class="sparkles">
        <path class="sparkle s1" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
        <path class="sparkle s2" d="M0 -8 L2 -2 L8 0 L2 2 L0 8 L-2 2 L-8 0 L-2 -2 Z" />
      </g>
    </g>
  </svg>
</div>

<style>
  .scout {
    --scale: 1;
    /* Grey/white husky-style coat — deliberately distinct from the
       cream-coloured Labrador look of the Dancing Dog frames pack, so
       the two don't read as "the same dog" in the character gallery. */
    --fur-light: #c3cdd6;
    --fur: #8a97a3;
    --fur-shadow: #5c6873;
    --fur-dark: #3d4650;
    --snout: #f5f3ef;
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

  /* Settle bounce: a quick overshoot-and-recover the instant walking
     stops and idle begins — selling the idea that actual weight just
     came to a stop, instead of the motion simply switching off. Runs
     once (not infinite) and re-triggers naturally each time the state
     class changes to idle, since Svelte swaps state-{sceneState} as a
     whole rather than toggling a flag. */
  .state-idle .body-squash {
    animation: settle-bounce 420ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
    transform-origin: 100px 190px;
  }
  @keyframes settle-bounce {
    0% {
      transform: scale(1.04, 0.94) translateY(2px);
    }
    60% {
      transform: scale(0.98, 1.03) translateY(-2px);
    }
    100% {
      transform: scale(1, 1) translateY(0);
    }
  }

  .ground-shadow {
    fill: rgba(0, 0, 0, 0.16);
    filter: blur(1.5px);
    transition:
      rx 180ms ease,
      opacity 180ms ease;
  }

  .body {
    fill: url(#scout-body-shade);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .saddle {
    fill: var(--fur-shadow);
    opacity: 0.4;
  }
  .body-highlight {
    fill: rgba(255, 255, 255, 0.35);
    filter: blur(2px);
  }
  .fur-tufts {
    fill: none;
    stroke: var(--fur-shadow);
    stroke-width: 1.5;
    stroke-linecap: round;
    opacity: 0.5;
  }
  .head {
    fill: url(#scout-head-shade);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .head-highlight {
    fill: rgba(255, 255, 255, 0.4);
    filter: blur(2px);
  }

  .upper-leg,
  .lower-leg rect {
    fill: url(#scout-leg-shade);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }
  .foot {
    fill: var(--fur);
    stroke: var(--fur-dark);
    stroke-width: 2;
  }

  .ear-outer {
    fill: var(--fur-dark);
  }
  .ear-inner {
    fill: #2b323a;
  }

  .snout {
    fill: url(#scout-snout-shade);
  }
  .patch {
    fill: var(--snout);
  }

  .tail {
    fill: none;
    stroke: var(--fur);
    stroke-width: 14;
    stroke-linecap: round;
  }
  .tail-root {
    transform-origin: 155px 140px;
  }

  .nose {
    fill: #2a1f14;
  }
  .nose-shine {
    fill: rgba(255, 255, 255, 0.55);
  }

  .blush {
    fill: var(--blush);
    opacity: 0.55;
  }

  .eye-white {
    fill: white;
    stroke: var(--fur-dark);
    stroke-width: 1.5;
  }

  .pupil {
    fill: #2a1f14;
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

  .brow {
    fill: none;
    stroke: var(--fur-dark);
    stroke-width: 3.5;
    stroke-linecap: round;
    transition: transform 220ms ease;
  }

  .mouth {
    fill: none;
    stroke: #6b4a2a;
    stroke-width: 3.5;
    stroke-linecap: round;
  }

  .tongue {
    fill: #ff8fa3;
    opacity: 0;
  }

  .tear {
    fill: #bfe6ff;
    opacity: 0;
  }

  .prop-ball {
    opacity: 0;
    transform-origin: 134px 128px;
  }
  .prop-ball circle {
    fill: #e8503a;
    stroke: #a6692a;
    stroke-width: 1.5;
  }
  .prop-ball path {
    fill: none;
    stroke: #a6692a;
    stroke-width: 1.5;
  }

  /* "Look far" pose (eye-rest reminder): a paw lifts to shield/peer near
     the brow, head tilts up and out as if focusing on something distant. */
  .prop-lookfar {
    opacity: 0;
    fill: var(--fur);
    stroke: var(--fur-dark);
    stroke-width: 1.5;
    transform-origin: 124px 88px;
  }
  .prop-lookfar.state-ask .prop-lookfar {
    opacity: 1;
    animation: lookfar-peer 1.6s ease-in-out infinite;
  }
  @keyframes lookfar-peer {
    0%,
    100% {
      transform: translate(0, 0) rotate(0deg);
    }
    50% {
      transform: translate(2px, -2px) rotate(-4deg);
    }
  }

  .sparkles {
    opacity: 0;
  }
  .sparkle {
    fill: #ffe14d;
  }
  .s1 {
    transform: translate(40px, 60px);
  }
  .s2 {
    transform: translate(160px, 80px);
  }

  /* --- Leg walk cycle: front-left+back-right swing one way, --- */
  /* --- front-right+back-left swing the other, true alternating gait --- */
  .leg {
    transition: transform 180ms ease;
  }

  .state-walk_in .leg-front-left,
  .state-walk_out .leg-front-left,
  .state-walk_in .leg-back-right,
  .state-walk_out .leg-back-right {
    animation: swing-forward calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .leg-front-right,
  .state-walk_out .leg-front-right,
  .state-walk_in .leg-back-left,
  .state-walk_out .leg-back-left {
    animation: swing-back calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes swing-forward {
    0%,
    100% {
      transform: rotate(-24deg) scaleY(1);
    }
    25% {
      transform: rotate(0deg) scaleY(0.93);
    }
    50% {
      transform: rotate(24deg) scaleY(1);
    }
  }
  @keyframes swing-back {
    0%,
    100% {
      transform: rotate(24deg) scaleY(1);
    }
    25% {
      transform: rotate(0deg) scaleY(0.93);
    }
    50% {
      transform: rotate(-24deg) scaleY(1);
    }
  }

  /* Knee bend: the lower segment folds back during the lift/forward-swing
     half of each leg's cycle (clearing the ground) and straightens again
     on the plant/push half — a rigid single-segment leg can only ever
     look like a swinging stick, the knee is what makes it read as a leg
     taking a step. */
  .state-walk_in .leg-front-left .lower-leg,
  .state-walk_out .leg-front-left .lower-leg,
  .state-walk_in .leg-back-right .lower-leg,
  .state-walk_out .leg-back-right .lower-leg {
    animation: knee-bend-a calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  .state-walk_in .leg-front-right .lower-leg,
  .state-walk_out .leg-front-right .lower-leg,
  .state-walk_in .leg-back-left .lower-leg,
  .state-walk_out .leg-back-left .lower-leg {
    animation: knee-bend-b calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes knee-bend-a {
    0%,
    100% {
      transform: rotate(10deg);
    }
    25% {
      transform: rotate(-28deg);
    }
    50% {
      transform: rotate(4deg);
    }
  }
  @keyframes knee-bend-b {
    0%,
    100% {
      transform: rotate(4deg);
    }
    25% {
      transform: rotate(-28deg);
    }
    50% {
      transform: rotate(10deg);
    }
  }

  .state-walk_in .body-svg,
  .state-walk_out .body-svg {
    animation:
      breathe 2.6s ease-in-out infinite,
      walk-bob calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes walk-bob {
    0%,
    50%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    25% {
      transform: translateY(-4.5px) rotate(-1.5deg);
    }
    75% {
      transform: translateY(-4.5px) rotate(1.5deg);
    }
  }

  /* Weight shift: the torso leans a few px toward whichever diagonal leg
     pair is currently planted, the way a real quadruped's centre of mass
     actually moves with its stride instead of travelling in a dead
     straight line while only the legs underneath do any work. */
  .state-walk_in .upper-body-group,
  .state-walk_out .upper-body-group {
    animation: weight-shift calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes weight-shift {
    0%,
    100% {
      transform: translateX(-2.5px);
    }
    50% {
      transform: translateX(2.5px);
    }
  }

  /* Squash & stretch on each footfall: the body compresses a touch the
     instant a diagonal leg pair plants, then rebounds — the single
     biggest lever for "alive cartoon" vs "shape sliding on a track". */
  .state-walk_in .body-squash,
  .state-walk_out .body-squash {
    animation: footfall-squash calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
    transform-origin: 100px 190px;
  }
  @keyframes footfall-squash {
    0%,
    50%,
    100% {
      transform: scale(1, 1);
    }
    8%,
    58% {
      transform: scale(1.05, 0.93);
    }
  }

  .state-walk_in .ground-shadow,
  .state-walk_out .ground-shadow {
    animation: shadow-pulse calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes shadow-pulse {
    0%,
    50%,
    100% {
      opacity: 0.16;
      rx: 54px;
    }
    25%,
    75% {
      opacity: 0.1;
      rx: 46px;
    }
  }

  /* Head nods out of phase with the body bob, the way a real quadruped's
     head bobs opposite its stride, instead of moving as one rigid block. */
  .head-group {
    transform-origin: 100px 148px;
  }
  .state-walk_in .head-group,
  .state-walk_out .head-group {
    animation: head-nod calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes head-nod {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    25% {
      transform: translateY(1px) rotate(-2deg);
    }
    50% {
      transform: translateY(-2.5px) rotate(0deg);
    }
    75% {
      transform: translateY(1px) rotate(2deg);
    }
  }

  .ear {
    transform-origin: 100px 85px;
  }
  .state-walk_in .ear,
  .state-walk_out .ear {
    animation: ear-flop calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
  }
  @keyframes ear-flop {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(5deg);
    }
  }

  /* Tail follow-through: lags a beat behind the head-nod/body-bob instead
     of running on its own clock, so it reads as something being dragged
     along by the body rather than an independent wiggling prop. */
  .state-walk_in .tail-root,
  .state-walk_out .tail-root {
    animation: tail-wag-walk calc(360ms / var(--anim-speed, 1)) ease-in-out infinite;
    animation-delay: -60ms;
  }

  /* States */
  .state-ask .tail-root {
    animation: tail-wag 900ms ease-in-out infinite;
  }
  .state-ask .prop-ball {
    opacity: 1;
    animation: ball-wiggle 1.4s ease-in-out infinite;
  }
  @keyframes tail-wag {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(14deg);
    }
  }
  @keyframes tail-wag-walk {
    0%,
    100% {
      transform: rotate(-9deg);
    }
    50% {
      transform: rotate(9deg);
    }
  }

  /* --- Reminder-specific ask poses --- */

  /* Look far (eye-rest): head tilts up and out, eyes widen slightly, as
     if focusing on something in the distance rather than the usual
     head-on "asking" look. */
  .prop-lookfar.state-ask .head-group {
    animation: lookfar-head-tilt 1.6s ease-in-out infinite;
  }
  @keyframes lookfar-head-tilt {
    0%,
    100% {
      transform: translateY(0) rotate(-6deg);
    }
    50% {
      transform: translateY(-2px) rotate(-10deg);
    }
  }
  .prop-lookfar.state-ask .eye-white {
    animation: lookfar-eyes-widen 1.6s ease-in-out infinite;
    transform-origin: center;
  }
  @keyframes lookfar-eyes-widen {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.12);
    }
  }

  /* Stretch: front legs reach up and out overhead, back arches slightly —
     echoes a real stretch rather than the neutral standing ask pose. */
  .prop-stretch.state-ask .leg-front-left {
    animation: stretch-leg-left 1.4s ease-in-out infinite;
    transform-origin: 70px 150px;
  }
  .prop-stretch.state-ask .leg-front-right {
    animation: stretch-leg-right 1.4s ease-in-out infinite;
    transform-origin: 130px 150px;
  }
  @keyframes stretch-leg-left {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-50deg) translate(2px, -6px);
    }
  }
  @keyframes stretch-leg-right {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(50deg) translate(-2px, -6px);
    }
  }
  .prop-stretch.state-ask .body-squash {
    animation: stretch-body-arch 1.4s ease-in-out infinite;
  }
  @keyframes stretch-body-arch {
    0%,
    100% {
      transform: scale(1, 1);
    }
    50% {
      transform: scale(1.05, 0.96);
    }
  }

  /* Posture: sits up a touch straighter and taller than the usual ask
     stance, head level and alert rather than tilted — echoes "sit up
     straight" rather than a generic asking pose. */
  .prop-posture.state-ask .upper-body-group {
    animation: posture-sit-tall 600ms ease-out both;
  }
  @keyframes posture-sit-tall {
    0% {
      transform: translateY(0) scaleY(1);
    }
    100% {
      transform: translateY(-5px) scaleY(1.03);
    }
  }
  .prop-posture.state-ask .head-group {
    animation: posture-head-level 600ms ease-out both;
  }
  @keyframes posture-head-level {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(-2deg);
    }
  }
  @keyframes ball-wiggle {
    0%,
    100% {
      transform: rotate(0deg) scale(1);
    }
    50% {
      transform: rotate(-10deg) scale(1.08);
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
  .state-happy .tongue {
    opacity: 1;
    animation: tongue-pant 260ms ease-in-out infinite;
  }
  @keyframes tongue-pant {
    0%,
    100% {
      transform: scaleY(1) translateY(0);
    }
    50% {
      transform: scaleY(1.2) translateY(2px);
    }
  }

  /* Stand up on the hind legs: the upper body (everything above the hip)
     rotates upright on top of the planted back legs, front legs lift off
     the ground and tuck in like paws held to the chest, head tips back
     with the tongue out — the single clearest "excited yes!" pose a
     flat 2D rig can give, closer to a real dog's reaction than any amount
     of extra bounce on the all-fours pose. */
  .upper-body-group {
    transform-origin: 100px 160px;
  }
  .state-happy .upper-body-group {
    /* Scout's silhouette is already drawn upright (head stacked above
       body above hips, teddy-bear-style, not a horizontal on-all-fours
       layout) — so standing "on two legs" doesn't rotate the torso at
       all, that previously tipped the whole body onto its side. The
       torso just rises a touch taller and settles, while the legs below
       do the actual work of lifting the front paws and bracing the back
       legs straight (see leg keyframes below). */
    animation: stand-up 900ms cubic-bezier(0.25, 1.1, 0.4, 1) both;
  }
  @keyframes stand-up {
    0% {
      transform: translateY(0) scaleY(1);
    }
    35% {
      transform: translateY(-14px) scaleY(1.04);
    }
    100% {
      transform: translateY(-11px) scaleY(1.03);
    }
  }

  /* Front legs lift off the ground and fold up to the chest like paws
     held up — hip rotates the whole leg up and inward, the knee folds
     the lower segment back underneath it. */
  .state-happy .leg-front-left {
    animation: paws-tuck-left 450ms ease-out both;
    transform-origin: 70px 150px;
  }
  .state-happy .leg-front-right {
    animation: paws-tuck-right 450ms ease-out both;
    transform-origin: 130px 150px;
  }
  @keyframes paws-tuck-left {
    0% {
      transform: rotate(0deg) translate(0, 0);
    }
    100% {
      transform: rotate(18deg) translate(4px, -22px);
    }
  }
  @keyframes paws-tuck-right {
    0% {
      transform: rotate(0deg) translate(0, 0);
    }
    100% {
      transform: rotate(-18deg) translate(-4px, -22px);
    }
  }
  .state-happy .leg-front-left .lower-leg,
  .state-happy .leg-front-right .lower-leg {
    animation: paw-fold 450ms ease-out both;
  }
  @keyframes paw-fold {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(-110deg);
    }
  }

  /* Back legs straighten fully vertical and stretch a touch taller to
     brace the torso's extra height while standing. */
  .state-happy .leg-back-left,
  .state-happy .leg-back-right {
    animation: back-legs-brace 450ms ease-out both;
  }
  @keyframes back-legs-brace {
    0% {
      transform: rotate(0deg) scaleY(1);
    }
    100% {
      transform: rotate(0deg) scaleY(1.12);
    }
  }
  .state-happy .leg-back-left .lower-leg,
  .state-happy .leg-back-right .lower-leg {
    animation: back-knee-brace 450ms ease-out both;
  }
  @keyframes back-knee-brace {
    0% {
      transform: rotate(10deg) scaleY(1);
    }
    100% {
      transform: rotate(0deg) scaleY(1.08);
    }
  }

  .state-happy .head-group {
    animation: happy-head-tilt-back 500ms ease-out both;
  }
  @keyframes happy-head-tilt-back {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(10deg) translateX(2px);
    }
  }

  .state-happy .tail-root {
    animation: tail-wag-fast 220ms ease-in-out infinite, tail-tuck-low 500ms ease-out both;
  }
  @keyframes tail-tuck-low {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(-20deg) translateY(6px);
    }
  }

  .state-happy .ground-shadow {
    animation: shadow-shrink 500ms ease-out both;
  }
  @keyframes shadow-shrink {
    0% {
      rx: 54px;
      opacity: 0.16;
    }
    100% {
      rx: 30px;
      opacity: 0.1;
    }
  }

  .state-happy .sparkles {
    opacity: 1;
    animation: sparkle-burst 700ms ease-out;
  }
  @keyframes tail-wag-fast {
    0%,
    100% {
      transform: rotate(-18deg);
    }
    50% {
      transform: rotate(18deg);
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

  .state-shrug .leg-front-left,
  .state-shrug .leg-front-right {
    animation: shrug-legs 600ms ease-out both;
  }
  @keyframes shrug-legs {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(-8px);
    }
  }

  .state-sad .ear {
    transform: translateY(6px) scale(0.92);
  }
  .state-sad .tail-root {
    transform: rotate(0deg) translateY(10px);
  }
  .state-sad .tear {
    opacity: 1;
    animation: tear-drop 1.4s ease-in infinite;
  }
  .state-sad .head-group {
    transform: translateY(4px) rotate(-3deg);
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

  .state-wave .leg-front-right {
    animation: wave 500ms ease-in-out 3;
    transform-origin: 130px 150px;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(-45deg);
    }
  }

  /* --- Procedural idle gestures: played once in a while during "idle" --- */
  .gesture-tilt.state-idle .head-group {
    animation: gesture-head-tilt 1000ms ease-in-out;
  }
  @keyframes gesture-head-tilt {
    0%,
    100% {
      transform: rotate(0deg);
    }
    35%,
    65% {
      transform: rotate(-12deg);
    }
  }

  .gesture-sniff.state-idle .head-group {
    animation: gesture-sniff 1000ms ease-in-out;
  }
  @keyframes gesture-sniff {
    0%,
    100% {
      transform: translateY(0) rotate(0deg);
    }
    30% {
      transform: translateY(10px) rotate(4deg);
    }
    50% {
      transform: translateY(11px) rotate(-3deg);
    }
    70% {
      transform: translateY(10px) rotate(4deg);
    }
  }
  .gesture-sniff.state-idle .nose {
    animation: gesture-nose-twitch 220ms ease-in-out 3;
  }
  @keyframes gesture-nose-twitch {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.2);
    }
  }

  .gesture-tail-chase.state-idle .body-svg {
    animation:
      breathe 2.6s ease-in-out infinite,
      gesture-spin 1400ms cubic-bezier(0.3, 0, 0.7, 1);
    transform-origin: 100px 160px;
  }
  @keyframes gesture-spin {
    0%,
    100% {
      transform: rotate(0deg);
    }
    50% {
      transform: rotate(360deg);
    }
  }
  .gesture-tail-chase.state-idle .tail-root {
    animation: tail-wag-fast 180ms ease-in-out infinite;
  }

  .dust-puffs {
    position: absolute;
    bottom: 3px;
    left: 50%;
    translate: -50% 0;
    width: 75px;
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
    animation: puff-rise 410ms ease-out infinite;
  }
  .p1 {
    left: 10px;
    animation-delay: 0ms;
  }
  .p2 {
    left: 32px;
    animation-delay: 135ms;
  }
  .p3 {
    left: 55px;
    animation-delay: 270ms;
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
