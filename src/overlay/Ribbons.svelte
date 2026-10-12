<script lang="ts">
  // A light celebratory burst for every YES — fewer, longer streamer
  // shapes than the full Confetti burst (which is reserved for the
  // bigger daily-goal-reached moment), radiating outward from the
  // character rather than falling from above.
  interface Streamer {
    id: number;
    angleDeg: number;
    distance: number; // px, how far it travels
    delayMs: number;
    durationMs: number;
    color: string;
    rotateStart: number;
    length: number; // px
  }

  const COLORS = ["#FFE14D", "#E8471C", "#3478F6", "#2AA37E", "#FF9EC7"];
  const STREAMER_COUNT = 10;

  let streamers: Streamer[] = Array.from({ length: STREAMER_COUNT }, (_, i) => {
    // Spread roughly upward/outward in a fan, not a full circle — this
    // bursts from behind the character like a party popper, not an
    // explosion in every direction.
    const spread = -70 + (140 / (STREAMER_COUNT - 1)) * i;
    return {
      id: i,
      angleDeg: spread + (Math.random() - 0.5) * 12,
      distance: 50 + Math.random() * 40,
      delayMs: Math.random() * 80,
      durationMs: 550 + Math.random() * 250,
      color: COLORS[i % COLORS.length],
      rotateStart: Math.random() * 360,
      length: 14 + Math.random() * 10,
    };
  });
</script>

<div class="ribbon-burst">
  {#each streamers as s (s.id)}
    <span
      class="streamer"
      style="
        background: {s.color};
        height: {s.length}px;
        animation-delay: {s.delayMs}ms;
        animation-duration: {s.durationMs}ms;
        --angle: {s.angleDeg}deg;
        --distance: {s.distance}px;
        --rotate-start: {s.rotateStart}deg;
      "
    ></span>
  {/each}
</div>

<style>
  .ribbon-burst {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    pointer-events: none;
    overflow: visible;
  }

  .streamer {
    position: absolute;
    bottom: 35%;
    width: 4px;
    border-radius: 2px;
    opacity: 0;
    animation-name: ribbon-burst;
    animation-timing-function: ease-out;
    animation-fill-mode: both;
    transform-origin: center;
  }

  @keyframes ribbon-burst {
    0% {
      opacity: 0;
      transform: rotate(var(--angle)) translateY(0) rotate(var(--rotate-start));
    }
    15% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: rotate(var(--angle)) translateY(calc(-1 * var(--distance)))
        rotate(calc(var(--rotate-start) + 180deg));
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .streamer {
      animation-duration: 10ms !important;
    }
  }
</style>
