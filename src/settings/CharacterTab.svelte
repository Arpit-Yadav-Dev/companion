<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type {
    CharacterPrefs,
    ThemeName,
    EntranceSidePref,
    PositionPref,
    PackInfo,
    Reminder,
    CatalogEntry,
    DownloadProgress,
  } from "../shared/types";
  import SvgCharacter from "../overlay/SvgCharacter.svelte";
  import { svgCharacterFor } from "../overlay/characterRegistry";
  import FramesCharacter from "../overlay/FramesCharacter.svelte";

  let prefs = $state<CharacterPrefs | null>(null);
  let packs = $state<PackInfo[]>([]);
  let reminders = $state<Reminder[]>([]);
  let catalog = $state<CatalogEntry[]>([]);
  let importing = $state(false);
  let importError = $state("");
  /** id -> 0-100, or null while a download has started but has no total yet. */
  let downloading = $state<Record<string, number | null>>({});
  let downloadError = $state<Record<string, string>>({});
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

  // Human-readable label for each known prop id, so the "Reminder poses"
  // section below reads as "Water -> Bottle" rather than raw JSON keys.
  const PROP_LABELS: Record<string, string> = {
    bottle: "Bottle",
    stretch: "Stretch",
    lookfar: "Look far",
    posture: "Sit up straight",
  };

  onMount(() => {
    void (async () => {
      [prefs, packs, reminders, catalog] = await Promise.all([
        invoke<CharacterPrefs>("get_character_prefs"),
        invoke<PackInfo[]>("list_packs"),
        invoke<Reminder[]>("get_reminders"),
        invoke<CatalogEntry[]>("list_catalog"),
      ]);
    })();

    const unlisten = listen<DownloadProgress>("pack://download-progress", (event) => {
      downloading = { ...downloading, [event.payload.id]: event.payload.percent };
    });
    return () => {
      void unlisten.then((off) => off());
    };
  });

  async function downloadCharacter(entry: CatalogEntry) {
    downloadError = { ...downloadError, [entry.id]: "" };
    downloading = { ...downloading, [entry.id]: null };
    try {
      await invoke<PackInfo>("download_pack", { id: entry.id });
      // Refresh both lists so the new character appears in the gallery
      // and drops out of the "available to download" section.
      [packs, catalog] = await Promise.all([
        invoke<PackInfo[]>("list_packs"),
        invoke<CatalogEntry[]>("list_catalog"),
      ]);
      selectPack(entry.id);
    } catch (e) {
      downloadError = { ...downloadError, [entry.id]: String(e) };
    } finally {
      const { [entry.id]: _removed, ...rest } = downloading;
      downloading = rest;
    }
  }

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

  let activePack = $derived(packs.find((p) => p.manifest.id === prefs?.activePackId) ?? null);
</script>

{#snippet characterPreview(pack: PackInfo, scale: number)}
  {@const Character = svgCharacterFor(pack.manifest.id)}
  {#if Character}
    <Character sceneState="idle" theme={prefs?.theme ?? "blue"} {scale} />
  {:else if pack.manifest.renderer === "svg"}
    <SvgCharacter sceneState="idle" theme={prefs?.theme ?? "blue"} {scale} />
  {:else if pack.manifest.renderer === "frames"}
    <FramesCharacter
      sceneState="idle"
      manifest={pack.manifest}
      packDir={pack.dir}
      {scale}
      maxFramesPerState={1}
    />
  {:else}
    <div class="placeholder">🖼️</div>
  {/if}
{/snippet}

<h1 class="panel-title">Character</h1>
<p class="panel-subtitle">Pick who shows up, and how.</p>

{#if prefs}
  {#if activePack}
    <section class="hero card">
      <div class="hero-preview">
        {@render characterPreview(activePack, 1)}
      </div>
      <div class="hero-info">
        <span class="hero-eyebrow">Currently active</span>
        <h2 class="hero-name">{activePack.manifest.name}</h2>
        <p class="hero-meta">
          {activePack.manifest.renderer === "frames" ? "Animated from video" : "Hand-drawn"} · {prefs.sizePercent}%
          size
        </p>
      </div>
    </section>
  {/if}

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
            {@render characterPreview(pack, 0.78)}
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

  {#if catalog.some((c) => !c.installed)}
    <section class="section">
      <h2 class="section-title">More characters</h2>
      <p class="section-subtitle">
        Animated characters, downloaded only if you want them — so the app stays small.
      </p>

      <div class="catalog">
        {#each catalog.filter((c) => !c.installed) as entry (entry.id)}
          <div class="catalog-card" class:busy={entry.id in downloading}>
            <div class="catalog-preview">
              <img src={entry.preview} alt={entry.name} />

              {#if entry.id in downloading}
                <div class="catalog-overlay">
                  <div
                    class="progress-bar"
                    class:indeterminate={downloading[entry.id] === null}
                    style="--pct: {downloading[entry.id] ?? 0}%"
                  ></div>
                  <span class="progress-label">
                    {downloading[entry.id] === null
                      ? "Starting…"
                      : `${downloading[entry.id]}%`}
                  </span>
                </div>
              {:else}
                <button class="catalog-download" onclick={() => downloadCharacter(entry)}>
                  <span class="download-icon" aria-hidden="true">↓</span>
                  <span>{entry.sizeMb} MB</span>
                </button>
              {/if}
            </div>

            <span class="catalog-name">{entry.name}</span>
            <span class="catalog-desc">{entry.description}</span>
            {#if downloadError[entry.id]}
              <span class="error">{downloadError[entry.id]}</span>
            {/if}
          </div>
        {/each}
      </div>
    </section>
  {/if}

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

      <label class="field">
        <span class="field-label">
          Animation speed — {prefs.animationSpeedPercent}%
        </span>
        <input
          type="range"
          min="25"
          max="150"
          step="5"
          bind:value={prefs.animationSpeedPercent}
          oninput={scheduleSave}
        />
      </label>
    </section>
  </div>

  <section class="section card pad">
    <h2 class="section-title">Reminder poses</h2>
    <p class="section-subtitle">
      Each reminder can give the character a fitting pose instead of the generic one — enable a
      reminder in the Reminders tab to see it in action.
    </p>
    <div class="pose-list">
      {#each reminders as reminder (reminder.id)}
        <div class="pose-row" class:dim={!reminder.enabled}>
          <span class="pose-emoji">{reminder.emoji}</span>
          <span class="pose-title">{reminder.title}</span>
          <span class="pose-arrow">→</span>
          <span class="pose-tag">{PROP_LABELS[reminder.prop ?? ""] ?? "Generic"}</span>
          {#if !reminder.enabled}
            <span class="pose-off">off</span>
          {/if}
        </div>
      {/each}
    </div>
  </section>
{/if}

<style>
  .hero {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: var(--space-5);
    margin-bottom: var(--space-5);
  }

  .hero-preview {
    flex: 0 0 auto;
    width: 110px;
    height: 110px;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .hero-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .hero-eyebrow {
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--accent);
  }

  .hero-name {
    margin: 0;
    font-size: 1.4rem;
    font-weight: 700;
    color: var(--text);
  }

  .hero-meta {
    margin: 0;
    font-size: 0.82rem;
    color: var(--text-muted);
  }

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

  .section-subtitle {
    margin: calc(var(--space-3) * -1) 0 var(--space-3) 0;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .import-btn {
    font-size: 0.78rem;
    padding: 0.4rem 0.8rem;
  }

  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(128px, 1fr));
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
    height: 92px;
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

  .catalog {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: var(--space-3);
  }

  .catalog-card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-3) var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    text-align: center;
  }

  .catalog-preview {
    position: relative;
    height: 96px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: var(--space-2);
  }

  .catalog-preview img {
    max-height: 100%;
    max-width: 100%;
    object-fit: contain;
  }

  /* The download control sits over the preview so the character itself
     is what the eye lands on first. */
  .catalog-download {
    position: absolute;
    inset: auto 0 0 50%;
    translate: -50% 0;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 0.3rem 0.7rem;
    border: none;
    border-radius: 999px;
    background: var(--accent);
    color: var(--accent-contrast);
    font: inherit;
    font-size: 0.74rem;
    font-weight: 650;
    cursor: pointer;
    box-shadow: var(--shadow-sm);
    transition:
      transform 0.1s ease,
      filter 0.15s ease;
  }
  .catalog-download:hover {
    filter: brightness(1.08);
    transform: translateY(-1px);
  }
  .catalog-download:active {
    transform: scale(0.97);
  }

  .download-icon {
    font-size: 0.85rem;
    line-height: 1;
  }

  .catalog-overlay {
    position: absolute;
    inset: auto var(--space-2) 0;
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .catalog-card.busy .catalog-preview img {
    opacity: 0.45;
  }

  .catalog-name {
    font-size: 0.82rem;
    font-weight: 650;
    color: var(--text);
  }

  .catalog-desc {
    font-size: 0.72rem;
    color: var(--text-muted);
    line-height: 1.3;
  }

  .progress-bar {
    position: relative;
    flex: 1;
    height: 6px;
    border-radius: 999px;
    background: var(--border);
    overflow: hidden;
  }
  .progress-bar::after {
    content: "";
    position: absolute;
    inset: 0 auto 0 0;
    width: var(--pct, 0%);
    background: var(--accent);
    border-radius: inherit;
    transition: width 0.2s ease;
  }
  /* No content-length from the server, so no real percentage to show —
     sweep a partial bar instead of pretending to know the progress. */
  .progress-bar.indeterminate::after {
    width: 35%;
    animation: progress-sweep 1.1s ease-in-out infinite;
  }
  @keyframes progress-sweep {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(285%);
    }
  }

  .progress-label {
    font-size: 0.7rem;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .pose-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .pose-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-1);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
  }
  .pose-row.dim {
    opacity: 0.45;
  }

  .pose-emoji {
    font-size: 1.1rem;
    width: 1.5rem;
    text-align: center;
  }

  .pose-title {
    font-weight: 600;
    color: var(--text);
    min-width: 7rem;
  }

  .pose-arrow {
    color: var(--text-faint);
  }

  .pose-tag {
    font-size: 0.75rem;
    font-weight: 650;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
  }

  .pose-off {
    margin-left: auto;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-faint);
  }
</style>
