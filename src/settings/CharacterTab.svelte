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
  import BiscuitCharacter from "../overlay/BiscuitCharacter.svelte";
  import SproutCharacter from "../overlay/SproutCharacter.svelte";
  import SunnyCharacter from "../overlay/SunnyCharacter.svelte";
  import MarshmallowCharacter from "../overlay/MarshmallowCharacter.svelte";
  import FramesCharacter from "../overlay/FramesCharacter.svelte";

  let prefs = $state<CharacterPrefs | null>(null);
  let packs = $state<PackInfo[]>([]);
  let importing = $state(false);
  let importError = $state("");
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  const THEMES: { value: ThemeName; label: string }[] = [
    { value: "blue", label: "Blue" },
    { value: "peach", label: "Peach" },
    { value: "mint", label: "Mint" },
    { value: "night", label: "Night" },
  ];
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

<h1 class="panel-title">Character</h1>
<p class="panel-subtitle">Pick who shows up, and how.</p>

{#if prefs}
  <section class="section">
    <div class="section-header">
      <h2 class="section-title">Gallery</h2>
      <button class="btn import-btn" onclick={importPack} disabled={importing}>
        {importing ? "Importing…" : "+ Import pack (.zip)"}
      </button>
    </div>

    <div class="gallery">
      {#each packs as pack (pack.manifest.id)}
        <button
          class="pack-card"
          class:active={prefs.activePackId === pack.manifest.id}
          onclick={() => selectPack(pack.manifest.id)}
        >
          <div class="preview">
            {#if pack.manifest.id === "pixel"}
              <PixelCharacter sceneState="idle" scale={0.72} />
            {:else if pack.manifest.id === "biscuit"}
              <BiscuitCharacter sceneState="idle" scale={0.72} />
            {:else if pack.manifest.id === "sprout"}
              <SproutCharacter sceneState="idle" scale={0.72} />
            {:else if pack.manifest.id === "sunny"}
              <SunnyCharacter sceneState="idle" scale={0.72} />
            {:else if pack.manifest.id === "marshmallow"}
              <MarshmallowCharacter sceneState="idle" scale={0.72} />
            {:else if pack.manifest.renderer === "svg"}
              <SvgCharacter sceneState="idle" theme={prefs.theme} scale={0.72} />
            {:else if pack.manifest.renderer === "frames"}
              <FramesCharacter sceneState="idle" manifest={pack.manifest} packDir={pack.dir} scale={0.3} />
            {:else}
              <div class="placeholder">🖼️</div>
            {/if}
          </div>
          <span class="pack-name">{pack.manifest.name}</span>
          {#if prefs.activePackId === pack.manifest.id}
            <span class="active-badge">Active</span>
          {/if}
        </button>
      {/each}
    </div>

    {#if importError}
      <p class="error">{importError}</p>
    {/if}
  </section>

  <div class="two-col">
    <section class="section card pad">
      <h2 class="section-title">Appearance</h2>

      <label class="field">
        <span class="field-label">Theme (applies to Sip)</span>
        <div class="theme-swatches">
          {#each THEMES as theme (theme.value)}
            <button
              class="swatch swatch-{theme.value}"
              class:active={prefs.theme === theme.value}
              onclick={() => {
                if (prefs) prefs.theme = theme.value;
                scheduleSave();
              }}
              aria-label={theme.label}
              title={theme.label}
            ></button>
          {/each}
        </div>
      </label>

      <label class="field">
        <span class="field-label">Size — {prefs.sizePercent}%</span>
        <input
          type="range"
          min="60"
          max="140"
          step="5"
          bind:value={prefs.sizePercent}
          oninput={scheduleSave}
        />
      </label>
    </section>

    <section class="section card pad">
      <h2 class="section-title">Movement</h2>

      <label class="field">
        <span class="field-label">Entrance side</span>
        <select bind:value={prefs.entranceSide} onchange={scheduleSave}>
          {#each ENTRANCE_OPTIONS as opt (opt.value)}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </label>

      <label class="field">
        <span class="field-label">Resting position</span>
        <select bind:value={prefs.position} onchange={scheduleSave}>
          {#each POSITION_OPTIONS as opt (opt.value)}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </label>
    </section>
  </div>
{/if}

<style>
  .section {
    margin-bottom: var(--space-6);
  }

  .section.pad {
    padding: var(--space-4) var(--space-5);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  .section-title {
    margin: 0 0 var(--space-3) 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text);
  }
  .section-header .section-title {
    margin-bottom: 0;
  }

  .import-btn {
    font-size: 0.78rem;
    padding: 0.4rem 0.8rem;
  }

  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(108px, 1fr));
    gap: var(--space-3);
  }

  .pack-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-2) var(--space-2);
    border-radius: var(--radius-md);
    border: 1.5px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    color: inherit;
    font: inherit;
    transition:
      border-color 0.12s ease,
      box-shadow 0.12s ease,
      transform 0.1s ease;
  }

  .pack-card:hover {
    border-color: var(--border-strong);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

  .pack-card.active {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .preview {
    height: 76px;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .placeholder {
    font-size: 2rem;
  }

  .pack-name {
    font-size: 0.78rem;
    font-weight: 650;
    color: var(--text);
  }

  .active-badge {
    position: absolute;
    top: 6px;
    right: 6px;
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--accent-contrast);
    background: var(--accent);
    padding: 0.15rem 0.4rem;
    border-radius: 999px;
  }

  .two-col {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: var(--space-4);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }
  .field:last-child {
    margin-bottom: 0;
  }

  .field select {
    width: 100%;
  }

  .theme-swatches {
    display: flex;
    gap: var(--space-2);
  }

  .swatch {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 0.1s ease;
  }
  .swatch:hover {
    transform: scale(1.08);
  }

  .swatch.active {
    border-color: var(--text);
    box-shadow: 0 0 0 2px var(--surface);
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

  .error {
    color: var(--danger);
    font-size: 0.8rem;
    margin-top: var(--space-2);
  }
</style>
