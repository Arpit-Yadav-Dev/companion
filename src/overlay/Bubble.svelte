<script lang="ts">
  interface Props {
    text: string;
    yesLabel: string;
    laterLabel: string;
    showButtons: boolean;
    onYes: () => void;
    onLater: () => void;
  }

  let { text, yesLabel, laterLabel, showButtons, onYes, onLater }: Props = $props();

  // Re-split into words whenever the text changes, so each new line gets its
  // own pop-in pass (keyed by the word's index + the text itself below).
  let words = $derived(text.split(" ").filter((w) => w.length > 0));
</script>

<div class="bubble">
  <p class="line">
    {#each words as word, i (text + i)}
      <span class="word" style="animation-delay: {i * 120}ms">{word}</span>{" "}
    {/each}
  </p>

  {#if showButtons}
    <div class="buttons">
      <button class="pill yes" onclick={onYes}>{yesLabel}</button>
      <button class="pill later" onclick={onLater}>{laterLabel}</button>
    </div>
  {/if}
</div>

<style>
  .bubble {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.9rem;
    transform: rotate(-2deg);
  }

  .line {
    margin: 0;
    text-align: center;
    font-family: "Baloo 2", "Comic Sans MS", "Arial Rounded MT Bold", sans-serif;
    font-weight: 800;
    font-size: 2rem;
    line-height: 1.2;
    max-width: 22ch;
  }

  .word {
    display: inline-block;
    color: #ffe14d;
    -webkit-text-stroke: 4px #e8471c;
    paint-order: stroke fill;
    text-shadow: 0 3px 6px rgba(0, 0, 0, 0.35);
    animation: word-pop 420ms cubic-bezier(0.34, 1.56, 0.64, 1) both;
  }

  @keyframes word-pop {
    0% {
      transform: scale(0.6);
      opacity: 0;
    }
    60% {
      transform: scale(1.08);
      opacity: 1;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  .buttons {
    display: flex;
    gap: 0.85rem;
    animation: buttons-fade-in 280ms ease-out 300ms both;
  }

  @keyframes buttons-fade-in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .pill {
    border: none;
    border-radius: 999px;
    padding: 0.65rem 1.5rem;
    font-size: 1rem;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
    background: white;
    color: #2b2b2b;
    box-shadow:
      0 3px 8px rgba(0, 0, 0, 0.2),
      inset 0 0 0 1.5px #e3e3e3;
    transition:
      transform 0.12s ease,
      box-shadow 0.12s ease;
  }

  .pill:hover {
    transform: translateY(-3px);
    box-shadow:
      0 6px 14px rgba(0, 0, 0, 0.25),
      inset 0 0 0 1.5px #d8d8d8;
  }

  .pill:active {
    transform: translateY(0) scale(0.95);
    box-shadow:
      0 2px 4px rgba(0, 0, 0, 0.2),
      inset 0 0 0 1.5px #d0d0d0;
  }

  .pill.yes {
    background: #ffe14d;
  }
</style>
