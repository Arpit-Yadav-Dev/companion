<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type {
    CharacterPrefs,
    ThemeName,
    EntranceSidePref,
    PositionPref,
    PackInfo,
  } from "../shared/types";
  import SvgCharacter from "../overlay/SvgCharacter.svelte";
  import PixelCharacter from "../overlay/PixelCharacter.svelte";

  let prefs = $state<CharacterPrefs | null>(null);
  let packs = $state<PackInfo[]>([]);
  let importing = $state(false);
  let importError = $state("");
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
    packs = await invoke<PackInfo[]>("list_packs");
  });

  function scheduleSave() {
    if (!prefs) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      invoke("save_character_prefs", { prefs });
    }, 300);
  }

  function selectPack(id: string) {
    if (!prefs) return;
    prefs.activePackId = id;
    scheduleSave();
  }

  async function importPack() {
    importing = true;
    importError = "";
    try {
      const result = await invoke<PackInfo | null>("import_pack");
      if (result) {
        packs = await invoke<PackInfo[]>("list_packs");
        selectPack(result.manifest.id);
      }
    } catch (e) {
      importError = String(e);
    } finally {
      importing = false;
    }
  }
</script>

<h2>Character</h2>

{#if prefs}
  <div class="gallery">
    {#each packs as pack (pack.manifest.id)}
      <button
        class="pack-card"
        class:active={prefs.activePackId === pack.manifest.id}
        onclick={() => selectPack(pack.manifest.id)}
      >
        <div class="preview">
          {#if pack.manifest.renderer === "svg" && pack.manifest.id === "pixel"}
            <PixelCharacter sceneState="idle" scale={prefs.sizePercent / 100} />
          {:else if pack.manifest.renderer === "svg"}
            <SvgCharacter sceneState="idle" theme={prefs.theme} scale={prefs.sizePercent / 100} />
          {:else}
            <div class="preview placeholder">🖼️</div>
          {/if}
        </div>
        <span class="pack-name">{pack.manifest.name}</span>
      </button>
    {/each}
  </div>

  <button class="import-btn" onclick={importPack} disabled={importing}>
    {importing ? "Importing…" : "Import pack (.zip)"}
  </button>
  {#if importError}
    <p class="error">{importError}</p>
  {/if}

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
    background: none;
    cursor: pointer;
    color: inherit;
    font: inherit;
  }

  .pack-card:hover {
    background: rgba(0, 0, 0, 0.04);
  }

  .pack-card.active {
    border-color: #3478f6;
    background: rgba(52, 120, 246, 0.08);
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

  .import-btn {
    margin-top: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: 1px dashed rgba(0, 0, 0, 0.25);
    background: none;
    cursor: pointer;
    font-size: 0.85rem;
    color: inherit;
  }

  .import-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .error {
    color: #e0432b;
    font-size: 0.8rem;
    margin-top: 0.4rem;
  }
</style>
