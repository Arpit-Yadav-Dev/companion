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
  const ENTRANCE_OPTIONS: { value: EntranceSidePref; label: string; short: string }[] = [
    { value: "random", label: "Random (left or right)", short: "Either side" },
    { value: "left", label: "Always from left", short: "Left" },
    { value: "right", label: "Always from right", short: "Right" },
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

  // The two renderers behave differently enough to be worth separating:
  // hand-drawn characters are tiny and always present, while video ones are
  // heavier and (once the roster is final) downloaded on demand. Grouping
  // them keeps that distinction obvious without the user needing to know
  // the word "renderer".
  let drawnPacks = $derived(packs.filter((p) => p.manifest.renderer !== "frames"));
  let videoPacks = $derived(packs.filter((p) => p.manifest.renderer === "frames"));

  // A bare percentage doesn't say much on its own — "80%" of what? Naming
  // the setting's effect means the number only has to confirm a choice the
  // label already made.
  function sizeLabel(percent: number): string {
    if (percent <= 75) return `Small · ${percent}%`;
    if (percent <= 110) return `Normal · ${percent}%`;
    return `Large · ${percent}%`;
  }

  function speedLabel(percent: number): string {
    if (percent <= 60) return `Calm · ${percent}%`;
    if (percent <= 115) return `Normal · ${percent}%`;
    return `Lively · ${percent}%`;
  }

  // Only a few reminders carry a pose, so listing every one of them meant
  // rows of "Generic" that said nothing. Show the ones that differ and
  // count the rest.
  let posedReminders = $derived(reminders.filter((r) => r.prop && PROP_LABELS[r.prop]));
  let genericCount = $derived(reminders.length - posedReminders.length);
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

{#snippet gallery(list: PackInfo[])}
  <div class="gallery">
    {#each list as pack (pack.manifest.id)}
      <button
        class="pack-card"
        class:active={prefs?.activePackId === pack.manifest.id}
        onclick={() => selectPack(pack.manifest.id)}
      >
        <div class="preview">
          {@render characterPreview(pack, 1)}
        </div>
        <span class="pack-name">{pack.manifest.name}</span>
        {#if prefs?.activePackId === pack.manifest.id}
          <span class="active-badge">Active</span>
        {/if}
      </button>
    {/each}
  </div>
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

    {#if drawnPacks.length}
      <div class="group-label">
        <span class="group-name">Illustrated</span>
        <span class="group-hint">Built in · tiny · {drawnPacks.length}</span>
      </div>
      {@render gallery(drawnPacks)}
    {/if}

    {#if videoPacks.length}
      <div class="group-label">
        <span class="group-name">Animated</span>
        <span class="group-hint">From video · {videoPacks.length}</span>
      </div>
      {@render gallery(videoPacks)}
    {/if}

    {#if importError}
      <p class="error">{importError}</p>
    {/if}
  </section>

  {#if catalog.some((c) => !c.installed)}
    <section class="section">
      <div class="group-label">
        <span class="group-name">More characters</span>
        <span class="group-hint">
          {catalog.filter((c) => !c.installed).length} available to download
        </span>
      </div>
      <p class="section-subtitle">
        Animated from video, fetched only if you pick them — which is what keeps the app
        small.
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

      <div class="field">
        <div class="slider-head">
          <span class="field-label">Size</span>
          <span class="slider-value">{sizeLabel(prefs.sizePercent)}</span>
        </div>
        <input
          type="range"
          min="60"
          max="140"
          step="5"
          bind:value={prefs.sizePercent}
          oninput={scheduleSave}
          aria-label="Character size"
        />
      </div>
    </section>

    <section class="section card pad">
      <h2 class="section-title">Movement</h2>

      <div class="field">
        <span class="field-label">Where it stands</span>
        <!-- A little screen diagram rather than a dropdown: the choice is
             about a place on screen, and showing that place is clearer
             (and quicker to scan) than reading three similar phrases. -->
        <div class="position-picker">
          {#each POSITION_OPTIONS as opt (opt.value)}
            <button
              class="position-option pos-{opt.value}"
              class:picked={prefs.position === opt.value}
              onclick={() => {
                if (prefs) prefs.position = opt.value;
                scheduleSave();
              }}
              title={opt.label}
              aria-label={opt.label}
              aria-pressed={prefs.position === opt.value}
            >
              <span class="screen"><span class="dot"></span></span>
            </button>
          {/each}
        </div>
      </div>

      <div class="field">
        <span class="field-label">Walks in from</span>
        <div class="chips">
          {#each ENTRANCE_OPTIONS as opt (opt.value)}
            <button
              class="chip"
              class:picked={prefs.entranceSide === opt.value}
              onclick={() => {
                if (prefs) prefs.entranceSide = opt.value;
                scheduleSave();
              }}
            >
              {opt.short}
            </button>
          {/each}
        </div>
      </div>

      <div class="field">
        <div class="slider-head">
          <span class="field-label">Animation speed</span>
          <span class="slider-value">{speedLabel(prefs.animationSpeedPercent)}</span>
        </div>
        <input
          type="range"
          min="25"
          max="150"
          step="5"
          bind:value={prefs.animationSpeedPercent}
          oninput={scheduleSave}
          aria-label="Animation speed"
        />
      </div>
    </section>
  </div>

  <section class="section card pad">
    <h2 class="section-title">Reminder poses</h2>
    <p class="section-subtitle">
      Some reminders give the character a fitting pose instead of its generic one.
    </p>
    {#if posedReminders.length}
      <div class="pose-tags">
        {#each posedReminders as reminder (reminder.id)}
          <span class="pose-tag" class:dim={!reminder.enabled}>
            <span class="pose-emoji">{reminder.emoji}</span>
            {reminder.title}
            <span class="pose-prop">{PROP_LABELS[reminder.prop ?? ""]}</span>
          </span>
        {/each}
      </div>
    {/if}
    <p class="pose-note">
      The other {genericCount} use the character's generic pose.
    </p>
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
    align-items: flex-end;
    justify-content: center;
    overflow: hidden;
  }

  /* Same fit-to-box treatment as the gallery cards: characters are taller
     than this tile, so scale them down rather than crop their heads. */
  .hero-preview :global(> *) {
    transform: scale(0.6);
    transform-origin: bottom center;
    flex: none;
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


  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
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

  .group-label {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    margin: var(--space-4) 0 var(--space-2);
  }

  .group-name {
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--text);
  }

  .group-hint {
    font-size: 0.72rem;
    color: var(--text-dim);
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
    height: 108px;
    width: 100%;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    overflow: hidden;
  }

  /* Characters render at their own natural height (~160-170px), which is
     taller than this box — previously the overflow simply cropped their
     heads and feet. Scale the rendered character down to fit instead, from
     the bottom so they stand on the card's floor rather than float. */
  .preview :global(> *) {
    transform: scale(var(--preview-fit, 0.62));
    transform-origin: bottom center;
    flex: none;
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

  .slider-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .slider-value {
    font-size: 0.76rem;
    font-weight: 650;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .chip {
    padding: 0.32rem 0.75rem;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 0.77rem;
    font-weight: 600;
    font-family: var(--font-body);
    cursor: pointer;
    transition:
      background 0.12s ease,
      color 0.12s ease,
      border-color 0.12s ease;
  }
  .chip:hover {
    color: var(--text);
    border-color: var(--text-faint);
  }
  .chip.picked {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-contrast);
  }

  /* Three small screens with a dot where the character would stand. */
  .position-picker {
    display: flex;
    gap: var(--space-2);
  }

  .position-option {
    padding: 0;
    border: none;
    background: none;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .screen {
    display: block;
    position: relative;
    width: 58px;
    height: 38px;
    border-radius: 5px;
    border: 1.5px solid var(--border-strong);
    background: var(--surface);
    transition:
      border-color 0.12s ease,
      background 0.12s ease;
  }

  .dot {
    position: absolute;
    bottom: 5px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--text-faint);
    transition: background 0.12s ease;
  }

  .pos-bottom_left .dot {
    left: 6px;
  }
  .pos-bottom_right .dot {
    right: 6px;
  }
  .pos-bottom_center .dot {
    left: 50%;
    translate: -50% 0;
  }

  .position-option:hover .screen {
    border-color: var(--text-faint);
  }
  .position-option.picked .screen {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .position-option.picked .dot {
    background: var(--accent);
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

  .pose-tags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .pose-tag {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.7rem 0.3rem 0.5rem;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    background: var(--surface-sunken);
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text);
  }
  .pose-tag.dim {
    opacity: 0.5;
  }

  .pose-emoji {
    font-size: 0.95rem;
  }

  .pose-prop {
    font-size: 0.72rem;
    font-weight: 650;
    color: var(--accent);
  }

  .pose-note {
    margin: var(--space-3) 0 0;
    font-size: 0.76rem;
    color: var(--text-muted);
  }
</style>
