<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let version = $state("");
  let confirmingReset = $state(false);

  onMount(async () => {
    version = await invoke<string>("app_version");
  });

  function openDataFolder() {
    invoke("open_data_folder");
  }

  async function resetAll() {
    if (!confirmingReset) {
      confirmingReset = true;
      return;
    }
    await invoke("reset_all_data");
    confirmingReset = false;
    // A full reload is the simplest way to get every tab's already-loaded
    // state back in sync with the now-empty stores.
    location.reload();
  }
</script>

<h1 class="panel-title">About</h1>
<p class="panel-subtitle">Version info, where your data lives, and how to start fresh.</p>

<section class="card pad identity-card">
  <!-- Same drawn mark as the sidebar rather than an emoji, so the app's
       own identity doesn't change between two places in one window. -->
  <span class="mark" aria-hidden="true">
    <svg viewBox="0 0 16 16" width="22" height="22">
      <path d="M8 1.6s4.3 4.8 4.3 7.6a4.3 4.3 0 0 1-8.6 0C3.7 6.4 8 1.6 8 1.6Z" fill="currentColor" />
    </svg>
  </span>
  <div>
    <span class="app-name">Companion</span>
    <span class="version">Version {version}</span>
  </div>
</section>

<section class="section card pad">
  <h2 class="section-title">Your data</h2>
  <p class="row-hint">
    Everything is stored locally as JSON — no accounts, no network calls.
  </p>
  <button class="btn" onclick={openDataFolder}>Open data folder</button>
</section>

<section class="section card pad danger-zone">
  <h2 class="section-title danger-title">Danger zone</h2>
  <p class="row-hint">Clears all reminders, preferences, and stats back to their defaults.</p>
  <div class="danger-actions">
    <button class="btn btn-danger" onclick={resetAll}>
      {confirmingReset ? "Click again to confirm" : "Reset all data"}
    </button>
    {#if confirmingReset}
      <button class="link" onclick={() => (confirmingReset = false)}>Cancel</button>
    {/if}
  </div>
</section>

<style>
  .identity-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  .mark {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 11px;
    background: var(--accent);
    color: var(--accent-contrast);
    box-shadow: var(--shadow-sm);
  }

  .app-name {
    display: block;
    font-family: var(--font-display);
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--text);
  }

  .version {
    display: block;
    font-size: 0.8rem;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    margin-top: 2px;
  }

  .section {
    margin-bottom: var(--space-4);
  }
  .section.pad {
    padding: var(--space-4) var(--space-5);
  }

  .section-title {
    margin: 0 0 var(--space-2) 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text);
  }

  .danger-title {
    color: var(--danger);
  }

  .row-hint {
    font-size: 0.82rem;
    color: var(--text-muted);
    margin: 0 0 var(--space-3) 0;
  }

  .danger-zone {
    border-color: color-mix(in srgb, var(--danger) 30%, var(--border));
  }

  .danger-actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .link {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
