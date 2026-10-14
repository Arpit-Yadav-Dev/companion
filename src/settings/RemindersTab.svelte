<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type {
    Reminder,
    DayOfWeek,
    PackInfo,
    CatalogEntry,
    CharacterPrefs,
    DownloadProgress,
  } from "../shared/types";
  import { svgCharacterFor } from "../overlay/characterRegistry";
  import SvgCharacter from "../overlay/SvgCharacter.svelte";
  import FramesCharacter from "../overlay/FramesCharacter.svelte";

  let reminders = $state<Reminder[]>([]);
  let installedPacks = $state<PackInfo[]>([]);
  let catalog = $state<CatalogEntry[]>([]);

  /** Every character the user could pick: the ones installed, plus the
   *  ones available to download. A reminder can name a character that
   *  isn't installed yet — it falls back until the user downloads it —
   *  so limiting this list to installed packs would hide exactly the
   *  character a reminder is best paired with. */
  let choosablePacks = $derived.by(() => {
    const seen = new Set<string>();
    const out: { id: string; name: string; installed: boolean; pack?: PackInfo }[] = [];
    for (const p of installedPacks) {
      seen.add(p.manifest.id);
      out.push({ id: p.manifest.id, name: p.manifest.name, installed: true, pack: p });
    }
    for (const c of catalog) {
      if (!seen.has(c.id)) out.push({ id: c.id, name: c.name, installed: false });
    }
    return out.sort((a, b) => a.name.localeCompare(b.name));
  });
  let expandedId = $state<string | null>(null);
  let testedId = $state<string | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let characterPrefs = $state<CharacterPrefs | null>(null);
  /** id -> 0-100, or null while a download has started but has no total. */
  let downloading = $state<Record<string, number | null>>({});
  let downloadFailed = $state<Record<string, string>>({});

  // Matches CATALOG_BASE_URL in src-tauri/src/packs.rs — previews for
  // characters that aren't installed come from the release, since their
  // artwork isn't on disk yet.
  const CATALOG_PREVIEW_BASE =
    "https://github.com/Arpit-Yadav-Dev/companion/releases/download/characters-v1";

  let defaultPack = $derived(
    installedPacks.find((p) => p.manifest.id === characterPrefs?.activePackId) ?? null,
  );

  const DAY_LABELS: { value: DayOfWeek; short: string }[] = [
    { value: 1, short: "M" },
    { value: 2, short: "T" },
    { value: 3, short: "W" },
    { value: 4, short: "T" },
    { value: 5, short: "F" },
    { value: 6, short: "S" },
    { value: 0, short: "S" },
  ];

  const WEEKDAYS: DayOfWeek[] = [1, 2, 3, 4, 5];
  const EVERY_DAY: DayOfWeek[] = [0, 1, 2, 3, 4, 5, 6];

  // Common intervals as one-tap chips. Typing an exact number still works,
  // but most people want a round value and shouldn't have to think in
  // minutes to get one.
  const INTERVAL_PRESETS = [15, 20, 30, 45, 60, 90, 120];
  const SNOOZE_PRESETS = [5, 10, 15, 30];

  // Whole-day windows people actually keep, so the common case is a click
  // rather than two time pickers.
  const HOURS_PRESETS: { label: string; start: number; end: number }[] = [
    { label: "Work day", start: 9 * 60, end: 18 * 60 },
    { label: "Waking hours", start: 8 * 60, end: 22 * 60 },
    { label: "All day", start: 0, end: 24 * 60 - 1 },
  ];

  const EMOJI_CHOICES = [
    "💧", "🧘", "👀", "🪑", "🚶", "🏋️", "☕", "🍎",
    "😮‍💨", "📵", "📖", "✏️", "🌿", "⏰",
  ];

  onMount(async () => {
    const unlisten = listen<DownloadProgress>("pack://download-progress", (event) => {
      downloading = { ...downloading, [event.payload.id]: event.payload.percent };
    });
    onDestroy(() => void unlisten.then((off) => off()));

    [reminders, installedPacks, catalog, characterPrefs] = await Promise.all([
      invoke<Reminder[]>("get_reminders"),
      invoke<PackInfo[]>("list_packs"),
      invoke<CatalogEntry[]>("list_catalog"),
      invoke<CharacterPrefs>("get_character_prefs"),
    ]);
  });

  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      invoke("save_reminders", { reminders });
    }, 300);
  }

  function toggleDay(reminder: Reminder, day: DayOfWeek) {
    const i = reminder.daysOfWeek.indexOf(day);
    if (i === -1) reminder.daysOfWeek = [...reminder.daysOfWeek, day].sort();
    else reminder.daysOfWeek = reminder.daysOfWeek.filter((d) => d !== day);
    scheduleSave();
  }

  function setDays(reminder: Reminder, days: DayOfWeek[]) {
    reminder.daysOfWeek = [...days].sort();
    scheduleSave();
  }

  function minutesToTimeInput(minutes: number): string {
    const h = Math.floor(minutes / 60)
      .toString()
      .padStart(2, "0");
    const m = (minutes % 60).toString().padStart(2, "0");
    return `${h}:${m}`;
  }

  function timeInputToMinutes(value: string): number {
    const [h, m] = value.split(":").map(Number);
    return h * 60 + m;
  }

  /** "Every 45 min · 9:00–18:00 · Weekdays" — so a collapsed card still
   *  answers the question the user opened this tab to ask. */
  function summarise(reminder: Reminder): string {
    const parts: string[] = [formatInterval(reminder.intervalMin)];

    const { startMinute, endMinute } = reminder.activeHours;
    const allDay = startMinute === 0 && endMinute >= 24 * 60 - 1;
    if (!allDay) {
      parts.push(`${formatClock(startMinute)}–${formatClock(endMinute)}`);
    }

    const days = [...reminder.daysOfWeek].sort();
    if (days.length === 7) parts.push("Every day");
    else if (days.length === 0) parts.push("No days selected");
    else if (days.join() === [...WEEKDAYS].sort().join()) parts.push("Weekdays");
    else if (days.join() === "0,6") parts.push("Weekends");
    else parts.push(days.map((d) => DAY_LABELS.find((x) => x.value === d)?.short).join(""));

    return parts.join(" · ");
  }

  function formatInterval(min: number): string {
    if (min < 60) return `Every ${min} min`;
    const h = min / 60;
    if (Number.isInteger(h)) return `Every ${h} hour${h === 1 ? "" : "s"}`;
    return `Every ${min} min`;
  }

  function formatClock(minutes: number): string {
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    const suffix = h < 12 ? "am" : "pm";
    const h12 = h % 12 === 0 ? 12 : h % 12;
    return m === 0 ? `${h12}${suffix}` : `${h12}:${m.toString().padStart(2, "0")}${suffix}`;
  }

  function addLine(reminder: Reminder) {
    reminder.lines = [...reminder.lines, ""];
    scheduleSave();
  }

  function removeLine(reminder: Reminder, index: number) {
    reminder.lines = reminder.lines.filter((_, i) => i !== index);
    scheduleSave();
  }

  function addCustomReminder() {
    const id = `custom-${Date.now()}`;
    const custom: Reminder = {
      id,
      enabled: true,
      emoji: "✏️",
      title: "Custom reminder",
      intervalMin: 60,
      activeHours: { startMinute: 9 * 60, endMinute: 18 * 60 },
      daysOfWeek: [...EVERY_DAY],
      lines: ["Your custom reminder text"],
      yesLabel: "Done!",
      laterLabel: "Remind me later",
      snoozeMin: 10,
      prop: null,
    };
    reminders = [...reminders, custom];
    expandedId = id;
    scheduleSave();
  }

  function removeReminder(id: string) {
    reminders = reminders.filter((r) => r.id !== id);
    scheduleSave();
  }

  /** Assigns a character to a reminder, fetching it first if it isn't
   *  installed. Picking a character that then silently doesn't appear is
   *  worse than a short wait, so the download happens here rather than
   *  sending the user to the Character tab to do it themselves. */
  async function chooseCharacter(reminder: Reminder, choice: { id: string; installed: boolean }) {
    reminder.characterPackId = choice.id;
    scheduleSave();
    if (choice.installed || choice.id in downloading) return;

    downloadFailed = { ...downloadFailed, [choice.id]: "" };
    downloading = { ...downloading, [choice.id]: null };
    try {
      await invoke<PackInfo>("download_pack", { id: choice.id });
      installedPacks = await invoke<PackInfo[]>("list_packs");
    } catch (e) {
      downloadFailed = { ...downloadFailed, [choice.id]: String(e) };
    } finally {
      const { [choice.id]: _done, ...rest } = downloading;
      downloading = rest;
    }
  }

  function testReminder(id: string) {
    invoke("trigger_test_reminder", { reminderId: id });
    // Confirm the click did something — the overlay appears on top of the
    // settings window, so without this the button can feel unresponsive.
    testedId = id;
    setTimeout(() => {
      if (testedId === id) testedId = null;
    }, 1600);
  }

  let enabledCount = $derived(reminders.filter((r) => r.enabled).length);
</script>

{#snippet castArt(pack: PackInfo)}
  {@const Character = svgCharacterFor(pack.manifest.id)}
  {#if Character}
    <Character sceneState="idle" theme={characterPrefs?.theme ?? "blue"} scale={1} />
  {:else if pack.manifest.renderer === "svg"}
    <SvgCharacter sceneState="idle" theme={characterPrefs?.theme ?? "blue"} scale={1} />
  {:else}
    <FramesCharacter
      sceneState="idle"
      manifest={pack.manifest}
      packDir={pack.dir}
      scale={1}
      maxFramesPerState={1}
    />
  {/if}
{/snippet}

<header class="panel-head">
  <div>
    <h1 class="panel-title">Reminders</h1>
    <p class="panel-subtitle">
      {enabledCount} of {reminders.length} active · click a card to edit its timing and wording.
    </p>
  </div>
  <button class="btn btn-primary" onclick={addCustomReminder}>+ New reminder</button>
</header>

<div class="list">
  {#each reminders as reminder (reminder.id)}
    {@const open = expandedId === reminder.id}
    <section class="card reminder" class:open class:off={!reminder.enabled}>
      <div class="card-header">
        <span class="emoji-badge" class:dim={!reminder.enabled}>{reminder.emoji}</span>

        <button
          class="heading"
          onclick={() => (expandedId = open ? null : reminder.id)}
          aria-expanded={open}
        >
          <span class="title">{reminder.title}</span>
          <span class="summary">{summarise(reminder)}</span>
        </button>

        <div class="header-actions">
          <button class="btn btn-small" onclick={() => testReminder(reminder.id)}>
            {testedId === reminder.id ? "Sent ✓" : "Test"}
          </button>

          <label class="switch" title={reminder.enabled ? "Turn off" : "Turn on"}>
            <input type="checkbox" bind:checked={reminder.enabled} onchange={scheduleSave} />
            <span class="track"><span class="thumb"></span></span>
          </label>

          <button
            class="chevron"
            class:up={open}
            onclick={() => (expandedId = open ? null : reminder.id)}
            aria-label={open ? "Collapse" : "Edit"}
          >
            <svg viewBox="0 0 12 8" width="12" height="8" aria-hidden="true">
              <path
                d="M1 1.5 6 6.5l5-5"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>
        </div>
      </div>

      {#if open}
        <div class="card-body">
          <div class="field">
            <span class="field-label">Name &amp; icon</span>
            <div class="name-row">
              <details class="emoji-picker">
                <summary class="emoji-current">{reminder.emoji}</summary>
                <div class="emoji-grid">
                  {#each EMOJI_CHOICES as choice (choice)}
                    <button
                      class="emoji-option"
                      class:picked={reminder.emoji === choice}
                      onclick={(e) => {
                        reminder.emoji = choice;
                        scheduleSave();
                        e.currentTarget.closest("details")?.removeAttribute("open");
                      }}
                    >
                      {choice}
                    </button>
                  {/each}
                </div>
              </details>
              <input type="text" bind:value={reminder.title} oninput={scheduleSave} />
            </div>
          </div>

          <div class="field">
            <span class="field-label">How often</span>
            <div class="chips">
              {#each INTERVAL_PRESETS as preset (preset)}
                <button
                  class="chip"
                  class:picked={reminder.intervalMin === preset}
                  onclick={() => {
                    reminder.intervalMin = preset;
                    scheduleSave();
                  }}
                >
                  {preset < 60 ? `${preset}m` : `${preset / 60}h`}
                </button>
              {/each}
              <span class="chip-custom">
                <input
                  type="number"
                  min="1"
                  bind:value={reminder.intervalMin}
                  oninput={scheduleSave}
                />
                <span class="unit">min</span>
              </span>
            </div>
          </div>

          <div class="field">
            <span class="field-label">Active hours</span>
            <div class="chips">
              {#each HOURS_PRESETS as preset (preset.label)}
                <button
                  class="chip"
                  class:picked={reminder.activeHours.startMinute === preset.start &&
                    reminder.activeHours.endMinute === preset.end}
                  onclick={() => {
                    reminder.activeHours.startMinute = preset.start;
                    reminder.activeHours.endMinute = preset.end;
                    scheduleSave();
                  }}
                >
                  {preset.label}
                </button>
              {/each}
            </div>
            <div class="time-row">
              <input
                type="time"
                value={minutesToTimeInput(reminder.activeHours.startMinute)}
                oninput={(e) => {
                  reminder.activeHours.startMinute = timeInputToMinutes(e.currentTarget.value);
                  scheduleSave();
                }}
              />
              <span class="dash">to</span>
              <input
                type="time"
                value={minutesToTimeInput(reminder.activeHours.endMinute)}
                oninput={(e) => {
                  reminder.activeHours.endMinute = timeInputToMinutes(e.currentTarget.value);
                  scheduleSave();
                }}
              />
            </div>
          </div>

          <div class="field">
            <span class="field-label">Days</span>
            <div class="days">
              {#each DAY_LABELS as day, i (i)}
                <button
                  class="day-pill"
                  class:active={reminder.daysOfWeek.includes(day.value)}
                  onclick={() => toggleDay(reminder, day.value)}
                  aria-pressed={reminder.daysOfWeek.includes(day.value)}
                >
                  {day.short}
                </button>
              {/each}
              <span class="day-shortcuts">
                <button class="link" onclick={() => setDays(reminder, EVERY_DAY)}>All</button>
                <button class="link" onclick={() => setDays(reminder, WEEKDAYS)}>Weekdays</button>
              </span>
            </div>
          </div>

          <div class="field">
            <span class="field-label">What it says</span>
            <p class="field-hint">
              One line is picked at random each time. Use
              <code>{"{name}"}</code>, <code>{"{count}"}</code> or <code>{"{streak}"}</code>.
            </p>
            {#each reminder.lines as _, i}
              <div class="line-row">
                <input
                  type="text"
                  bind:value={reminder.lines[i]}
                  oninput={scheduleSave}
                  placeholder="Say something…"
                />
                {#if reminder.lines.length > 1}
                  <button
                    class="icon-btn"
                    onclick={() => removeLine(reminder, i)}
                    aria-label="Remove line">✕</button
                  >
                {/if}
              </div>
            {/each}
            <button class="link" onclick={() => addLine(reminder)}>+ Add another line</button>
          </div>

          <div class="field-row">
            <label class="field">
              <span class="field-label">Yes button</span>
              <input type="text" bind:value={reminder.yesLabel} oninput={scheduleSave} />
            </label>
            <label class="field">
              <span class="field-label">Later button</span>
              <input type="text" bind:value={reminder.laterLabel} oninput={scheduleSave} />
            </label>
          </div>

          <div class="field">
            <span class="field-label">Snooze for</span>
            <div class="chips">
              {#each SNOOZE_PRESETS as preset (preset)}
                <button
                  class="chip"
                  class:picked={reminder.snoozeMin === preset}
                  onclick={() => {
                    reminder.snoozeMin = preset;
                    scheduleSave();
                  }}
                >
                  {preset}m
                </button>
              {/each}
              <span class="chip-custom">
                <input type="number" min="1" bind:value={reminder.snoozeMin} oninput={scheduleSave} />
                <span class="unit">min</span>
              </span>
            </div>
          </div>

          <div class="field">
            <span class="field-label">Who delivers it</span>
            <p class="field-hint">
              "Default" uses whichever character is picked in the Character tab.
            </p>
            <div class="cast-grid">
              <button
                class="cast-option"
                class:picked={!reminder.characterPackId}
                onclick={() => {
                  reminder.characterPackId = null;
                  scheduleSave();
                }}
              >
                <span class="cast-art">
                  {#if defaultPack}
                    {@render castArt(defaultPack)}
                  {/if}
                </span>
                <span class="cast-name">Default</span>
              </button>

              {#each choosablePacks as choice (choice.id)}
                <button
                  class="cast-option"
                  class:picked={reminder.characterPackId === choice.id}
                  class:not-installed={!choice.installed}
                  onclick={() => chooseCharacter(reminder, choice)}
                  title={choice.installed ? choice.name : `${choice.name} — download in the Character tab`}
                >
                  <span class="cast-art">
                    {#if choice.pack}
                      {@render castArt(choice.pack)}
                    {:else}
                      <img
                        class="cast-thumb"
                        src="{CATALOG_PREVIEW_BASE}/preview-{choice.id}.webp"
                        alt=""
                        loading="lazy"
                      />
                    {/if}
                  </span>
                  <span class="cast-name">{choice.name}</span>
                  {#if choice.id in downloading}
                    <span class="cast-badge busy" aria-label="Downloading">
                      {downloading[choice.id] ?? ""}
                    </span>
                  {:else if !choice.installed}
                    <span class="cast-badge" aria-label="Downloads when picked">↓</span>
                  {/if}
                </button>
              {/each}
            </div>
            {#if reminder.characterPackId && downloadFailed[reminder.characterPackId]}
              <p class="field-hint failed">
                Couldn't download {reminder.characterPackId} — this reminder uses the
                default until it succeeds. Check your connection and pick it again.
              </p>
            {/if}
          </div>

          {#if reminder.id.startsWith("custom-")}
            <div class="card-footer">
              <button class="btn btn-danger btn-small" onclick={() => removeReminder(reminder.id)}>
                Delete this reminder
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </section>
  {/each}
</div>

<style>
  .panel-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .reminder {
    overflow: hidden;
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }

  .reminder.open {
    border-color: var(--border-strong);
    box-shadow: var(--shadow-md);
  }

  /* A disabled reminder stays readable but visibly recedes, so the list
     reads at a glance as "these are on, those are off". */
  .reminder.off .title,
  .reminder.off .summary {
    color: var(--text-faint);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
  }

  .emoji-badge {
    flex: none;
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 9px;
    background: var(--accent-soft);
    font-size: 1.05rem;
  }

  .emoji-badge.dim {
    background: var(--surface-hover);
    filter: grayscale(1);
    opacity: 0.6;
  }

  .heading {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
    align-items: flex-start;
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .title {
    font-weight: 650;
    font-size: 0.9rem;
    color: var(--text);
  }

  .summary {
    font-size: 0.76rem;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .header-actions {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .chevron {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    transition:
      rotate 0.18s ease,
      background 0.12s ease;
  }
  .chevron:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .chevron.up {
    rotate: 180deg;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    border-top: 1px solid var(--border);
    background: var(--surface-sunken);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
  }

  .field-hint {
    margin: 0;
    font-size: 0.76rem;
    color: var(--text-muted);
  }

  .cast-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(74px, 1fr));
    gap: var(--space-2);
  }

  .cast-option {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: var(--space-2) 4px 6px;
    border-radius: var(--radius-md);
    border: 1.5px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    font: inherit;
    color: inherit;
    transition:
      border-color 0.12s ease,
      background 0.12s ease;
  }
  .cast-option:hover {
    border-color: var(--text-faint);
  }
  .cast-option.picked {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  /* Characters draw at their own natural height, which is taller than
     this tile; scale them to fit rather than cropping heads and feet. */
  .cast-art {
    height: 52px;
    width: 100%;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    overflow: hidden;
  }
  .cast-art :global(> *) {
    transform: scale(0.3);
    transform-origin: bottom center;
    flex: none;
  }

  .cast-thumb {
    max-height: 52px;
    width: auto;
    object-fit: contain;
  }

  .cast-name {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text);
    text-align: center;
    line-height: 1.2;
  }

  /* A character that still needs downloading is dimmed and flagged, so
     picking one doesn't silently fall back to the default. */
  .cast-option.not-installed .cast-art {
    opacity: 0.55;
  }
  .field-hint.failed {
    color: var(--danger);
  }

  .cast-badge {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 15px;
    height: 15px;
    border-radius: 50%;
    background: var(--accent);
    color: var(--accent-contrast);
    font-size: 0.6rem;
    font-weight: 700;
    display: grid;
    place-items: center;
  }

  /* Wider while a percentage is showing than it is as a bare arrow. */
  .cast-badge.busy {
    width: auto;
    min-width: 15px;
    padding: 0 4px;
    border-radius: var(--radius-pill);
    font-size: 0.55rem;
  }

  .field-hint code {
    font-size: 0.72rem;
    padding: 1px 4px;
    border-radius: 4px;
    background: var(--surface-hover);
  }

  .name-row {
    display: flex;
    gap: var(--space-2);
  }
  .name-row input[type="text"] {
    flex: 1;
  }

  .emoji-picker {
    position: relative;
  }
  .emoji-current {
    display: grid;
    place-items: center;
    width: 38px;
    height: 36px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    cursor: pointer;
    list-style: none;
    font-size: 1.05rem;
  }
  .emoji-current::-webkit-details-marker {
    display: none;
  }
  .emoji-grid {
    position: absolute;
    z-index: 5;
    top: calc(100% + 4px);
    left: 0;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
    padding: var(--space-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    background: var(--surface);
    box-shadow: var(--shadow-lg);
  }
  .emoji-option {
    width: 30px;
    height: 30px;
    border: none;
    border-radius: 6px;
    background: none;
    font-size: 1rem;
    cursor: pointer;
  }
  .emoji-option:hover {
    background: var(--surface-hover);
  }
  .emoji-option.picked {
    background: var(--accent-soft);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    align-items: center;
  }

  .chip {
    padding: 0.32rem 0.7rem;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 0.78rem;
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

  .chip-custom {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .chip-custom input {
    width: 64px;
    padding: 0.3rem 0.45rem;
    font-size: 0.78rem;
  }
  .unit {
    font-size: 0.74rem;
    color: var(--text-muted);
  }

  .time-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .dash {
    font-size: 0.78rem;
    color: var(--text-muted);
  }

  .days {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
  }

  .day-pill {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 0.76rem;
    font-weight: 650;
    font-family: var(--font-body);
    cursor: pointer;
    transition:
      background 0.12s ease,
      color 0.12s ease,
      border-color 0.12s ease;
  }
  .day-pill:hover {
    border-color: var(--text-faint);
    color: var(--text);
  }
  .day-pill.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-contrast);
  }

  .day-shortcuts {
    display: flex;
    gap: var(--space-3);
    margin-left: var(--space-2);
  }

  .line-row {
    display: flex;
    gap: var(--space-2);
  }
  .line-row input {
    flex: 1;
  }

  .icon-btn {
    width: 34px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.78rem;
  }
  .icon-btn:hover {
    color: var(--danger);
    border-color: var(--danger);
  }

  .link {
    align-self: flex-start;
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

  .btn-small {
    padding: 0.35rem 0.75rem;
    font-size: 0.78rem;
  }

  .card-footer {
    display: flex;
    justify-content: flex-end;
    padding-top: var(--space-2);
    border-top: 1px solid var(--border);
  }
</style>
