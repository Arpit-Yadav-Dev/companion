<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { GeneralPrefs } from "../shared/types";

  let prefs = $state<GeneralPrefs | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

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

    <label class="field">
      <span class="field-label">Language</span>
      <select disabled>
        <option>English</option>
      </select>
    </label>
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
        <span class="toggle-hint">A soft pop on entry, a cheerful ding on yes.</span>
      </span>
      <span class="switch">
        <input type="checkbox" bind:checked={prefs.soundEnabled} onchange={scheduleSave} />
        <span class="track"><span class="thumb"></span></span>
      </span>
    </label>

    {#if prefs.soundEnabled}
      <label class="field volume-field">
        <span class="field-label">Volume</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          bind:value={prefs.soundVolume}
          oninput={scheduleSave}
        />
      </label>
    {/if}
  </section>
{/if}

<style>
  .section {
    margin-bottom: var(--space-4);
  }
  .section.pad {
    padding: var(--space-4) var(--space-5);
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
  .field select {
    width: 100%;
  }

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
    margin-top: var(--space-1);
    max-width: 280px;
  }
</style>
