<script lang="ts">
  interface Piece {
    id: number;
    left: number; // percent
    delayMs: number;
    durationMs: number;
    color: string;
    rotateStart: number;
    drift: number; // px, horizontal sway
  }

  const COLORS = ["#FFE14D", "#E8471C", "#3478F6", "#2AA37E", "#FF9EC7"];
  const PIECE_COUNT = 36;

  let pieces: Piece[] = Array.from({ length: PIECE_COUNT }, (_, i) => ({
    id: i,
    left: Math.random() * 100,
    delayMs: Math.random() * 300,
    durationMs: 1100 + Math.random() * 500,
    color: COLORS[i % COLORS.length],
    rotateStart: Math.random() * 360,
    drift: (Math.random() - 0.5) * 80,
  }));
</script>

<div class="confetti-burst">
  {#each pieces as piece (piece.id)}
    <span
      class="piece"
      style="
        left: {piece.left}%;
        background: {piece.color};
        animation-delay: {piece.delayMs}ms;
        animation-duration: {piece.durationMs}ms;
        --rotate-start: {piece.rotateStart}deg;
        --drift: {piece.drift}px;
      "
    ></span>
  {/each}
</div>

<style>
  .confetti-burst {
    position: absolute;
    inset: -40px -40px auto -40px;
    height: 0;
    pointer-events: none;
    overflow: visible;
  }

  .piece {
    position: absolute;
    top: 0;
    width: 8px;
    height: 12px;
    border-radius: 2px;
    opacity: 0;
    animation-name: confetti-fall;
    animation-timing-function: ease-in;
    animation-fill-mode: both;
  }

  @keyframes confetti-fall {
    0% {
      opacity: 0;
      transform: translateY(-10px) translateX(0) rotate(var(--rotate-start));
    }
    10% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: translateY(140px) translateX(var(--drift)) rotate(calc(var(--rotate-start) + 360deg));
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .piece {
      animation-duration: 10ms !important;
    }
  }
</style>
