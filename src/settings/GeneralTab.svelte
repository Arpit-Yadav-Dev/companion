<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { GeneralPrefs } from "../shared/types";

  let prefs = $state<GeneralPrefs | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let previewing = $state(false);

  // Named levels rather than only a slider: "quiet enough for the office"
  // is the decision people actually make, and a bare 0-100 track makes
  // them guess which number means that.
  const VOLUME_PRESETS = [
    { label: "Quiet", value: 0.25 },
    { label: "Medium", value: 0.55 },
    { label: "Loud", value: 0.9 },
  ];

  function setVolume(value: number) {
    if (!prefs) return;
    prefs.soundVolume = value;
    scheduleSave();
    playPreview(value);
  }

  function previewSound() {
    if (prefs) playPreview(prefs.soundVolume);
  }

  /** Plays the YES sound at a given level, so the slider can be set by ear
   *  instead of by guessing what a percentage sounds like. */
  function playPreview(volume: number) {
    const audio = new Audio("/sounds/yes-clap.wav");
    audio.volume = Math.min(Math.max(volume, 0), 1);
    previewing = true;
    audio.addEventListener("ended", () => (previewing = false));
    void audio.play().catch(() => (previewing = false));
  }

  onMount(async () => {
    prefs = await invoke<GeneralPrefs>("get_general_prefs");
  });

  function scheduleSave() {
    if (!prefs) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      invoke("save_general_prefs", { prefs });
    }, 300);
  }
</script>

<h1 class="panel-title">General</h1>
<p class="panel-subtitle">Your name, startup behavior, and how the character moves and sounds.</p>

{#if prefs}
  <section class="section card pad">
    <h2 class="section-title">About you</h2>
    <label class="field">
      <span class="field-label">Your name</span>
      <input type="text" bind:value={prefs.userName} oninput={scheduleSave} placeholder="there" />
    </label>
    <p class="hint">Used wherever reminders say "{"{name}"}" — e.g. "Hey, {prefs.userName}".</p>

  </section>

  <section class="section card pad">
    <h2 class="section-title">Startup &amp; behavior</h2>
    <label class="toggle-row">
      <span>
        <span class="toggle-label">Launch at login</span>
        <span class="toggle-hint">Start Companion automatically when you log in.</span>
      </span>
      <span class="switch">
        <input type="checkbox" bind:checked={prefs.launchAtLogin} onchange={scheduleSave} />
        <span class="track"><span class="thumb"></span></span>
      </span>
    </label>

    <label class="toggle-row">
      <span>
        <span class="toggle-label">Start paused</span>
        <span class="toggle-hint">Don't show any reminders until you hit Resume.</span>
      </span>
      <span class="switch">
        <input type="checkbox" bind:checked={prefs.startPaused} onchange={scheduleSave} />
        <span class="track"><span class="thumb"></span></span>
      </span>
    </label>

    <label class="toggle-row">
      <span>
        <span class="toggle-label">Do Not Disturb</span>
        <span class="toggle-hint">Silences all reminders until you turn this off.</span>
      </span>
      <span class="switch">
        <input type="checkbox" bind:checked={prefs.dnd} onchange={scheduleSave} />
        <span class="track"><span class="thumb"></span></span>
      </span>
    </label>
  </section>

  <section class="section card pad">
    <h2 class="section-title">Character feel</h2>
    <label class="toggle-row">
      <span>
        <span class="toggle-label">Reduced motion</span>
        <span class="toggle-hint">Crossfade in/out instead of walking across the screen.</span>
      </span>
      <span class="switch">
        <input type="checkbox" bind:checked={prefs.reducedMotion} onchange={scheduleSave} />
        <span class="track"><span class="thumb"></span></span>
      </span>
    </label>

    <label class="toggle-row">
      <span>
        <span class="toggle-label">Sound effects</span>
        <span class="toggle-hint">A soft pop on entry, applause on yes.</span>
      </span>
      <span class="switch">
        <input type="checkbox" bind:checked={prefs.soundEnabled} onchange={scheduleSave} />
        <span class="track"><span class="thumb"></span></span>
      </span>
    </label>

    {#if prefs.soundEnabled}
      <div class="field volume-field">
        <div class="volume-head">
          <span class="field-label">Volume</span>
          <span class="volume-value">{Math.round(prefs.soundVolume * 100)}%</span>
        </div>

        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          bind:value={prefs.soundVolume}
          oninput={scheduleSave}
          aria-label="Sound volume"
        />

        <div class="volume-presets">
          {#each VOLUME_PRESETS as preset (preset.label)}
            <button
              class="chip"
              class:picked={Math.abs(prefs.soundVolume - preset.value) < 0.03}
              onclick={() => setVolume(preset.value)}
            >
              {preset.label}
            </button>
          {/each}
          <button class="link preview" onclick={previewSound}>
            {previewing ? "Playing…" : "▸ Preview"}
          </button>
        </div>

        <p class="toggle-hint">
          Quiet suits a shared office; the preview plays at the level you've set.
        </p>
      </div>
    {/if}
  </section>
{/if}

<style>
  .section {
    margin-bottom: var(--space-4);
  }

  .section-title {
    margin: 0 0 var(--space-4) 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
    max-width: 360px;
  }
  .field:last-child {
    margin-bottom: 0;
  }

  .field input[type="text"],
  .hint {
    font-size: 0.78rem;
    color: var(--text-faint);
    margin: calc(var(--space-2) * -1) 0 var(--space-3) 0;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) 0;
    border-top: 1px solid var(--border);
  }
  .toggle-row:first-of-type {
    border-top: none;
    padding-top: 0;
  }
  .toggle-row:last-of-type {
    padding-bottom: 0;
  }

  .toggle-label {
    display: block;
    font-size: 0.88rem;
    font-weight: 600;
    color: var(--text);
  }

  .toggle-hint {
    display: block;
    font-size: 0.76rem;
    color: var(--text-faint);
    margin-top: 2px;
  }

  .volume-field {
    margin-top: var(--space-2);
    max-width: 340px;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .volume-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .volume-value {
    font-size: 0.78rem;
    font-weight: 650;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .volume-presets {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .chip {
    padding: 0.3rem 0.7rem;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 0.76rem;
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

  .link {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 0.78rem;
    font-weight: 600;
    font-family: var(--font-body);
    cursor: pointer;
    padding: 0;
  }
  .link:hover {
    text-decoration: underline;
  }

  .preview {
    margin-left: auto;
  }
</style>
