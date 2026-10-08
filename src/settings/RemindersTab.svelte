<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { Reminder, DayOfWeek } from "../shared/types";

  let reminders = $state<Reminder[]>([]);
  let expandedId = $state<string | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  const DAY_LABELS: { value: DayOfWeek; short: string }[] = [
    { value: 0, short: "Su" },
    { value: 1, short: "Mo" },
    { value: 2, short: "Tu" },
    { value: 3, short: "We" },
    { value: 4, short: "Th" },
    { value: 5, short: "Fr" },
    { value: 6, short: "Sa" },
  ];

  onMount(async () => {
    reminders = await invoke<Reminder[]>("get_reminders");
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
      activeHours: { startMinute: 8 * 60, endMinute: 22 * 60 },
      daysOfWeek: [0, 1, 2, 3, 4, 5, 6],
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

  function testReminder(id: string) {
    invoke("trigger_test_reminder", { reminderId: id });
  }
</script>

<h1 class="panel-title">Reminders</h1>
<p class="panel-subtitle">Turn reminders on, tune their timing, and customize what they say.</p>

<div class="list">
  {#each reminders as reminder (reminder.id)}
    <div class="card">
      <div class="card-header">
        <label class="row">
          <input type="checkbox" bind:checked={reminder.enabled} onchange={scheduleSave} />
          <span class="emoji-badge">{reminder.emoji}</span>
          <span class="title">{reminder.title}</span>
        </label>
        <div class="header-actions">
          <button class="link" onclick={() => testReminder(reminder.id)}>Test</button>
          <button
            class="link"
            onclick={() => (expandedId = expandedId === reminder.id ? null : reminder.id)}
          >
            {expandedId === reminder.id ? "Collapse" : "Edit"}
          </button>
          {#if reminder.id.startsWith("custom-")}
            <button class="link danger" onclick={() => removeReminder(reminder.id)}>Remove</button>
          {/if}
        </div>
      </div>

      {#if expandedId === reminder.id}
        <div class="card-body">
          <label class="field">
            <span class="field-label">Interval (minutes)</span>
            <input
              type="number"
              min="1"
              bind:value={reminder.intervalMin}
              oninput={scheduleSave}
            />
          </label>

          <div class="field-row">
            <label class="field">
              <span class="field-label">Active from</span>
              <input
                type="time"
                value={minutesToTimeInput(reminder.activeHours.startMinute)}
                oninput={(e) => {
                  reminder.activeHours.startMinute = timeInputToMinutes(e.currentTarget.value);
                  scheduleSave();
                }}
              />
            </label>
            <label class="field">
              <span class="field-label">Active until</span>
              <input
                type="time"
                value={minutesToTimeInput(reminder.activeHours.endMinute)}
                oninput={(e) => {
                  reminder.activeHours.endMinute = timeInputToMinutes(e.currentTarget.value);
                  scheduleSave();
                }}
              />
            </label>
          </div>

          <div class="field">
            <span class="field-label">Days</span>
            <div class="days">
              {#each DAY_LABELS as day (day.value)}
                <button
                  class="day-pill"
                  class:active={reminder.daysOfWeek.includes(day.value)}
                  onclick={() => toggleDay(reminder, day.value)}
                >
                  {day.short}
                </button>
              {/each}
            </div>
          </div>

          <div class="field">
            <span class="field-label">Lines (one picked at random each time)</span>
            {#each reminder.lines as _, i}
              <div class="line-row">
                <input
                  type="text"
                  bind:value={reminder.lines[i]}
                  oninput={scheduleSave}
                  placeholder="Line text — supports {'{name}'}, {'{count}'}, {'{streak}'}"
                />
                {#if reminder.lines.length > 1}
                  <button class="icon-btn" onclick={() => removeLine(reminder, i)}>✕</button>
                {/if}
              </div>
            {/each}
            <button class="link" onclick={() => addLine(reminder)}>+ Add line</button>
          </div>

          <div class="field-row">
            <label class="field">
              <span class="field-label">YES button label</span>
              <input type="text" bind:value={reminder.yesLabel} oninput={scheduleSave} />
            </label>
            <label class="field">
              <span class="field-label">Later button label</span>
              <input type="text" bind:value={reminder.laterLabel} oninput={scheduleSave} />
            </label>
          </div>

          <label class="field">
            <span class="field-label">Snooze minutes</span>
            <input type="number" min="1" bind:value={reminder.snoozeMin} oninput={scheduleSave} />
          </label>
        </div>
      {/if}
    </div>
  {/each}
</div>

<button class="btn add-custom" onclick={addCustomReminder}>+ Custom reminder</button>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }

  .card {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    overflow: hidden;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .emoji-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: 8px;
    background: var(--accent-soft);
    font-size: 1rem;
  }

  .title {
    font-weight: 650;
    font-size: 0.88rem;
    color: var(--text);
  }

  .header-actions {
    display: flex;
    gap: var(--space-4);
  }

  .link {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
  }

  .link.danger {
    color: var(--danger);
  }

  .card-body {
    padding: var(--space-4);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    background: var(--surface-hover);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field input[type="text"],
  .field input[type="number"],
  .field input[type="time"] {
    width: 100%;
  }

  .field-row {
    display: flex;
    gap: var(--space-4);
    flex-wrap: wrap;
  }
  .field-row .field {
    flex: 1;
    min-width: 140px;
  }

  .days {
    display: flex;
    gap: var(--space-2);
  }

  .day-pill {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
    color: var(--text-muted);
    transition:
      background 0.12s ease,
      color 0.12s ease,
      border-color 0.12s ease;
  }
  .day-pill:hover {
    border-color: var(--accent);
  }

  .day-pill.active {
    background: var(--accent);
    color: var(--accent-contrast);
    border-color: var(--accent);
  }

  .line-row {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
  .line-row input {
    flex: 1;
    min-width: 0;
  }

  .icon-btn {
    flex-shrink: 0;
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--danger);
    font-size: 0.75rem;
  }
  .icon-btn:hover {
    background: color-mix(in srgb, var(--danger) 8%, transparent);
  }

  .add-custom {
    border-style: dashed;
    font-weight: 600;
  }
</style>
