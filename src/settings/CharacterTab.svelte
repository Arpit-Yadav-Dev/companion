<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { CharacterPrefs, ThemeName, EntranceSidePref, PositionPref } from "../shared/types";
  import SvgCharacter from "../overlay/SvgCharacter.svelte";

  let prefs = $state<CharacterPrefs | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  const THEMES: ThemeName[] = ["blue", "peach", "mint", "night"];
  const ENTRANCE_OPTIONS: { value: EntranceSidePref; label: string }[] = [
    { value: "random", label: "Random (left or right)" },
    { value: "left", label: "Always from left" },
    { value: "right", label: "Always from right" },
  ];
  const POSITION_OPTIONS: { value: PositionPref; label: string }[] = [
    { value: "bottom_left", label: "Bottom-left" },
    { value: "bottom_right", label: "Bottom-right" },
    { value: "bottom_center", label: "Bottom-center" },
  ];

  onMount(async () => {
    prefs = await invoke<CharacterPrefs>("get_character_prefs");
  });

  function scheduleSave() {
    if (!prefs) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      invoke("save_character_prefs", { prefs });
    }, 300);
  }
</script>

<h2>Character</h2>

{#if prefs}
  <div class="gallery">
    <div class="pack-card active">
      <div class="preview" style="--theme-scale: {prefs.sizePercent / 100}">
        <SvgCharacter sceneState="idle" theme={prefs.theme} scale={prefs.sizePercent / 100} />
      </div>
      <span class="pack-name">Sip</span>
    </div>
    <div class="pack-card disabled">
      <div class="preview placeholder">🐱</div>
      <span class="pack-name">Pixel (Phase 6)</span>
    </div>
  </div>

  <label class="field">
    <span>Theme</span>
    <div class="theme-swatches">
      {#each THEMES as theme (theme)}
        <button
          class="swatch swatch-{theme}"
          class:active={prefs.theme === theme}
          onclick={() => {
            if (prefs) prefs.theme = theme;
            scheduleSave();
          }}
          aria-label={theme}
        ></button>
      {/each}
    </div>
  </label>

  <label class="field">
    <span>Size: {prefs.sizePercent}%</span>
    <input
      type="range"
      min="60"
      max="140"
      step="5"
      bind:value={prefs.sizePercent}
      oninput={scheduleSave}
    />
  </label>

  <label class="field">
    <span>Entrance side</span>
    <select bind:value={prefs.entranceSide} onchange={scheduleSave}>
      {#each ENTRANCE_OPTIONS as opt (opt.value)}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </label>

  <label class="field">
    <span>Position</span>
    <select bind:value={prefs.position} onchange={scheduleSave}>
      {#each POSITION_OPTIONS as opt (opt.value)}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </label>

  <p class="hint">Import pack (.zip) arrives in Phase 6 once the frames renderer exists.</p>
{/if}

<style>
  h2 {
    margin-top: 0;
  }

  .gallery {
    display: flex;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .pack-card {
    width: 120px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    padding: 0.75rem;
    border-radius: 10px;
    border: 2px solid transparent;
  }

  .pack-card.active {
    border-color: #3478f6;
    background: rgba(52, 120, 246, 0.08);
  }

  .pack-card.disabled {
    opacity: 0.4;
  }

  .preview {
    height: 90px;
    display: flex;
    align-items: center;
    justify-content: center;
    transform: scale(0.5);
    transform-origin: center;
  }

  .preview.placeholder {
    font-size: 2.5rem;
    transform: none;
  }

  .pack-name {
    font-size: 0.8rem;
    font-weight: 600;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    margin-bottom: 1rem;
    max-width: 320px;
  }

  .field span {
    font-size: 0.85rem;
    font-weight: 600;
  }

  .field select {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    font-size: 0.9rem;
  }

  .theme-swatches {
    display: flex;
    gap: 0.5rem;
  }

  .swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
  }

  .swatch.active {
    border-color: #1d1d1f;
  }
  @media (prefers-color-scheme: dark) {
    .swatch.active {
      border-color: white;
    }
  }

  .swatch-blue {
    background: #6fc3ff;
  }
  .swatch-peach {
    background: #ffb199;
  }
  .swatch-mint {
    background: #8de9c6;
  }
  .swatch-night {
    background: #6b7bdb;
  }

  .hint {
    font-size: 0.8rem;
    opacity: 0.6;
  }
</style>
