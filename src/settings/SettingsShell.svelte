<script lang="ts">
  import "./theme.css";
  import GeneralTab from "./GeneralTab.svelte";
  import RemindersTab from "./RemindersTab.svelte";
  import CharacterTab from "./CharacterTab.svelte";
  import StatsTab from "./StatsTab.svelte";
  import AboutTab from "./AboutTab.svelte";

  type TabId = "general" | "reminders" | "character" | "stats" | "about";

  // Icons are inline stroke paths rather than emoji: emoji render at the
  // mercy of the system font, sit on their own baseline and can't take the
  // active colour, which is what made the old sidebar look unfinished.
  const TABS: { id: TabId; label: string; path: string }[] = [
    {
      id: "general",
      label: "General",
      path: "M8 10.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Zm5.4-1.6.9.7a.6.6 0 0 1 .1.8l-.9 1.5a.6.6 0 0 1-.7.3l-1.1-.4a4.6 4.6 0 0 1-1 .6l-.2 1.2a.6.6 0 0 1-.6.4H8.1a.6.6 0 0 1-.6-.4l-.2-1.2a4.6 4.6 0 0 1-1-.6l-1.1.4a.6.6 0 0 1-.7-.3l-.9-1.5a.6.6 0 0 1 .1-.8l.9-.7a4.8 4.8 0 0 1 0-1.2l-.9-.7a.6.6 0 0 1-.1-.8l.9-1.5a.6.6 0 0 1 .7-.3l1.1.4a4.6 4.6 0 0 1 1-.6l.2-1.2a.6.6 0 0 1 .6-.4h1.8a.6.6 0 0 1 .6.4l.2 1.2c.36.15.69.35 1 .6l1.1-.4a.6.6 0 0 1 .7.3l.9 1.5a.6.6 0 0 1-.1.8l-.9.7a4.8 4.8 0 0 1 0 1.2Z",
    },
    {
      id: "reminders",
      label: "Reminders",
      path: "M8 1.8a4.2 4.2 0 0 0-4.2 4.2c0 2.6-1.1 3.9-1.1 3.9h10.6s-1.1-1.3-1.1-3.9A4.2 4.2 0 0 0 8 1.8Zm1.5 10.3a1.6 1.6 0 0 1-3 0",
    },
    {
      id: "character",
      label: "Character",
      path: "M8 9.2a5 5 0 0 0-5 4.6h10a5 5 0 0 0-5-4.6Zm0-1.1a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z",
    },
    {
      id: "stats",
      label: "Stats",
      path: "M3 13V7.5M8 13V3M13 13v-4",
    },
    {
      id: "about",
      label: "About",
      path: "M8 14A6 6 0 1 0 8 2a6 6 0 0 0 0 12Zm0-8.6v.2M8 7.6V11",
    },
  ];

  let active = $state<TabId>("general");
</script>

<div class="shell">
  <nav class="tabs" aria-label="Settings sections">
    <div class="brand">
      <span class="brand-mark" aria-hidden="true">
        <svg viewBox="0 0 16 16" width="17" height="17">
          <g fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round">
            <path d="M5.4 4.3 3.4 2.2M10.6 4.3l2-2.1" />
          </g>
          <circle cx="3.1" cy="1.9" r="1.1" fill="currentColor" />
          <circle cx="12.9" cy="1.9" r="1.1" fill="currentColor" />
          <circle cx="8" cy="9.1" r="5.6" fill="currentColor" />
          <circle cx="6.1" cy="8.2" r="1.35" fill="var(--accent)" />
          <circle cx="9.9" cy="8.2" r="1.35" fill="var(--accent)" />
          <path d="M6.3 11.1q1.7 1.3 3.4 0" fill="none" stroke="var(--accent)"
                stroke-width="0.9" stroke-linecap="round" />
        </svg>
      </span>
      <span class="brand-name">Companion</span>
    </div>

    {#each TABS as tab (tab.id)}
      <button
        class="tab"
        class:active={active === tab.id}
        onclick={() => (active = tab.id)}
        aria-current={active === tab.id ? "page" : undefined}
      >
        <svg class="tab-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path
            d={tab.path}
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>{tab.label}</span>
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
    width: 198px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-right: 1px solid var(--border);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-2) var(--space-5);
  }

  .brand-mark {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 7px;
    background: var(--accent);
    color: var(--accent-contrast);
    box-shadow: var(--shadow-sm);
  }

  .brand-name {
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 0.95rem;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  .tab {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    text-align: left;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.65rem;
    font-size: 0.85rem;
    font-weight: 550;
    font-family: var(--font-body);
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }

  .tab-icon {
    flex: none;
    opacity: 0.85;
  }

  .tab:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  /* The selected row reads as a raised surface with an accent marker rather
     than a solid accent block — it keeps the sidebar calm when one of five
     rows is always selected, and keeps the label at full contrast. */
  .tab.active {
    background: var(--surface);
    color: var(--text);
    font-weight: 650;
    box-shadow: var(--shadow-sm);
  }

  .tab.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 50%;
    translate: 0 -50%;
    width: 3px;
    height: 16px;
    border-radius: var(--radius-pill);
    background: var(--accent);
  }

  .tab.active .tab-icon {
    opacity: 1;
    color: var(--accent);
  }

  .panel {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: var(--space-6) var(--space-6) var(--space-6);
  }
</style>
