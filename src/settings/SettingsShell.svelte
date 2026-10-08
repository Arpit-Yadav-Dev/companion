<script lang="ts">
  import "./theme.css";
  import GeneralTab from "./GeneralTab.svelte";
  import RemindersTab from "./RemindersTab.svelte";
  import CharacterTab from "./CharacterTab.svelte";
  import StatsTab from "./StatsTab.svelte";
  import AboutTab from "./AboutTab.svelte";

  type TabId = "general" | "reminders" | "character" | "stats" | "about";

  const TABS: { id: TabId; label: string; icon: string }[] = [
    { id: "general", label: "General", icon: "⚙️" },
    { id: "reminders", label: "Reminders", icon: "💧" },
    { id: "character", label: "Character", icon: "🎭" },
    { id: "stats", label: "Stats", icon: "📊" },
    { id: "about", label: "About", icon: "ℹ️" },
  ];

  let active = $state<TabId>("general");
</script>

<div class="shell">
  <nav class="tabs">
    <div class="brand">
      <span class="brand-mark">💧</span>
      <span class="brand-name">Companion</span>
    </div>
    {#each TABS as tab (tab.id)}
      <button class="tab" class:active={active === tab.id} onclick={() => (active = tab.id)}>
        <span class="tab-icon">{tab.icon}</span>
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
  .shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .tabs {
    width: 190px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-3);
    background: var(--surface);
    border-right: 1px solid var(--border);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-2) var(--space-4);
    margin-bottom: var(--space-2);
  }

  .brand-mark {
    font-size: 1.3rem;
  }

  .brand-name {
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 0.95rem;
    color: var(--text);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    text-align: left;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.65rem;
    font-size: 0.85rem;
    font-weight: 600;
    font-family: var(--font-body);
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .tab-icon {
    font-size: 0.95rem;
    line-height: 1;
  }

  .tab:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .tab.active {
    background: var(--accent);
    color: var(--accent-contrast);
  }

  .panel {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: var(--space-6) var(--space-6) var(--space-5);
  }
</style>
