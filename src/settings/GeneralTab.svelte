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

{#if prefs}
  <h2>General</h2>

  <label class="field">
    <span>Your name</span>
    <input type="text" bind:value={prefs.userName} oninput={scheduleSave} placeholder="there" />
  </label>
  <p class="hint">Used wherever reminders say "{"{name}"}" — e.g. "Hey, {prefs.userName}".</p>

  <label class="row">
    <input type="checkbox" bind:checked={prefs.launchAtLogin} onchange={scheduleSave} />
    <span>Launch at login</span>
  </label>

  <label class="row">
    <input type="checkbox" bind:checked={prefs.startPaused} onchange={scheduleSave} />
    <span>Start paused</span>
  </label>

  <label class="row">
    <input type="checkbox" bind:checked={prefs.dnd} onchange={scheduleSave} />
    <span>Do Not Disturb</span>
  </label>

  <label class="row">
    <input type="checkbox" bind:checked={prefs.reducedMotion} onchange={scheduleSave} />
    <span>Reduced motion (crossfade instead of walking)</span>
  </label>

  <label class="row">
    <input type="checkbox" bind:checked={prefs.soundEnabled} onchange={scheduleSave} />
    <span>Sound effects</span>
  </label>

  {#if prefs.soundEnabled}
    <label class="field">
      <span>Volume</span>
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

  <label class="field">
    <span>Language</span>
    <select disabled>
      <option>English</option>
    </select>
  </label>
{/if}

<style>
  h2 {
    margin-top: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    margin-bottom: 1rem;
    max-width: 360px;
  }

  .field span {
    font-size: 0.85rem;
    font-weight: 600;
  }

  .field input[type="text"],
  .field select {
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    font-size: 0.9rem;
  }

  .hint {
    font-size: 0.8rem;
    opacity: 0.6;
    margin-top: -0.6rem;
    margin-bottom: 1rem;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    font-size: 0.9rem;
  }
</style>
