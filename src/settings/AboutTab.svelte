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

<h2>About</h2>

<p class="version">Companion v{version}</p>

<button class="action" onclick={openDataFolder}>Open data folder</button>

<div class="danger-zone">
  <button class="action danger" onclick={resetAll}>
    {confirmingReset ? "Click again to confirm reset" : "Reset all data"}
  </button>
  {#if confirmingReset}
    <button class="link" onclick={() => (confirmingReset = false)}>Cancel</button>
  {/if}
  <p class="hint">Clears all reminders, preferences, and stats back to defaults.</p>
</div>

<style>
  h2 {
    margin-top: 0;
  }

  .version {
    font-size: 0.9rem;
    opacity: 0.7;
  }

  .action {
    display: block;
    margin-bottom: 0.75rem;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    background: white;
    font-size: 0.85rem;
    cursor: pointer;
    color: inherit;
  }

  @media (prefers-color-scheme: dark) {
    .action {
      background: #2a2a2a;
      border-color: rgba(255, 255, 255, 0.15);
    }
  }

  .action.danger {
    color: #e0432b;
    border-color: #e0432b;
  }

  .danger-zone {
    margin-top: 2rem;
  }

  .link {
    background: none;
    border: none;
    color: #3478f6;
    font-size: 0.82rem;
    cursor: pointer;
    margin-left: 0.5rem;
  }

  .hint {
    font-size: 0.8rem;
    opacity: 0.6;
    margin-top: 0.5rem;
  }
</style>
