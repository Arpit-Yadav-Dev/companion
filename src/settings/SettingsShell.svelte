<script lang="ts">
  import GeneralTab from "./GeneralTab.svelte";
  import RemindersTab from "./RemindersTab.svelte";
  import CharacterTab from "./CharacterTab.svelte";
  import StatsTab from "./StatsTab.svelte";
  import AboutTab from "./AboutTab.svelte";

  type TabId = "general" | "reminders" | "character" | "stats" | "about";

  const TABS: { id: TabId; label: string }[] = [
    { id: "general", label: "General" },
    { id: "reminders", label: "Reminders" },
    { id: "character", label: "Character" },
    { id: "stats", label: "Stats" },
    { id: "about", label: "About" },
  ];

  let active = $state<TabId>("general");
</script>

<div class="shell">
  <nav class="tabs">
    {#each TABS as tab (tab.id)}
      <button class="tab" class:active={active === tab.id} onclick={() => (active = tab.id)}>
        {tab.label}
      </button>
    {/each}
  </nav>

  <div class="panel">
    {#if active === "general"}
      <GeneralTab />
    {:else if active === "reminders"}
      <RemindersTab />
    {:else if active === "character"}
      <CharacterTab />
    {:else if active === "stats"}
      <StatsTab />
    {:else if active === "about"}
      <AboutTab />
    {/if}
  </div>
</div>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    font-family:
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      sans-serif;
  }

  :global(body) {
    background: #f5f5f7;
    color: #1d1d1f;
  }

  @media (prefers-color-scheme: dark) {
    :global(body) {
      background: #1e1e1e;
      color: #f0f0f0;
    }
  }

  .shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .tabs {
    width: 160px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 1rem 0.5rem;
    border-right: 1px solid rgba(0, 0, 0, 0.1);
  }

  @media (prefers-color-scheme: dark) {
    .tabs {
      border-right-color: rgba(255, 255, 255, 0.12);
    }
  }

  .tab {
    text-align: left;
    background: none;
    border: none;
    border-radius: 8px;
    padding: 0.5rem 0.75rem;
    font-size: 0.9rem;
    font-weight: 500;
    color: inherit;
    cursor: pointer;
  }

  .tab:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  .tab.active {
    background: #3478f6;
    color: white;
  }

  @media (prefers-color-scheme: dark) {
    .tab:hover {
      background: rgba(255, 255, 255, 0.08);
    }
  }

  .panel {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem 2rem;
  }
</style>
