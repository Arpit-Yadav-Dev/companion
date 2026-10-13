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
  <!-- The same character as the app icon. It used to be a water droplet,
       which made an app that also handles posture, medicine and leaving
       work look like it only tracked water. -->
  <span class="mark" aria-hidden="true">
    <svg viewBox="0 0 16 16" width="22" height="22">
      <circle cx="8" cy="8" r="7" fill="currentColor" />
          <circle cx="5.6" cy="6.4" r="1.15" fill="var(--accent)" />
          <circle cx="10.4" cy="6.4" r="1.15" fill="var(--accent)" />
          <path d="M5.3 9.4a3.4 3.4 0 0 0 5.4 0" fill="none" stroke="var(--accent)"
                stroke-width="1.15" stroke-linecap="round" />
    </svg>
  </span>
  <div>
    <span class="app-name">Companion</span>
    <span class="version">Version {version}</span>
  </div>
</section>

<section class="section card pad">
  <h2 class="section-title">Your data</h2>
  <!-- The app's privacy claim is worth stating plainly rather than in one
       passing clause: it's a reason to trust something that watches when
       you're at your desk. -->
  <ul class="facts">
    <li>Stored on this Mac only, as plain JSON</li>
    <li>No account, no sign-in</li>
    <li>Works offline — the app never calls home</li>
  </ul>
  <button class="btn" onclick={openDataFolder}>Open data folder</button>
</section>

<section class="section card pad danger-zone">
  <h2 class="section-title danger-title">Start fresh</h2>
  <p class="row-hint">
    Clears every reminder, preference and stat back to its default. This can't be undone.
  </p>
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


  .danger-title {
    color: var(--danger);
  }

  .row-hint {
    font-size: 0.82rem;
    color: var(--text-muted);
    margin: 0 0 var(--space-3) 0;
  }

  .facts {
    list-style: none;
    margin: 0 0 var(--space-4);
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .facts li {
    position: relative;
    padding-left: 1.3rem;
    font-size: 0.83rem;
    color: var(--text-muted);
  }

  /* A drawn tick rather than an emoji or a list bullet, so it takes the
     accent colour and sits on the text's own baseline. */
  .facts li::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0.42em;
    width: 9px;
    height: 5px;
    border-left: 2px solid var(--success);
    border-bottom: 2px solid var(--success);
    rotate: -45deg;
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
