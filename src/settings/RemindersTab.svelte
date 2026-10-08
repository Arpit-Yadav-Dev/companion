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

<h2>Reminders</h2>

<div class="list">
  {#each reminders as reminder (reminder.id)}
    <div class="card">
      <div class="card-header">
        <label class="row">
          <input type="checkbox" bind:checked={reminder.enabled} onchange={scheduleSave} />
          <span class="emoji">{reminder.emoji}</span>
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
            <span>Interval (minutes)</span>
            <input
              type="number"
              min="1"
              bind:value={reminder.intervalMin}
              oninput={scheduleSave}
            />
          </label>

          <div class="field-row">
            <label class="field">
              <span>Active from</span>
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
              <span>Active until</span>
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
            <span>Days</span>
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
            <span>Lines (one picked at random each time)</span>
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
              <span>YES button label</span>
              <input type="text" bind:value={reminder.yesLabel} oninput={scheduleSave} />
            </label>
            <label class="field">
              <span>Later button label</span>
              <input type="text" bind:value={reminder.laterLabel} oninput={scheduleSave} />
            </label>
          </div>

          <label class="field">
            <span>Snooze minutes</span>
            <input type="number" min="1" bind:value={reminder.snoozeMin} oninput={scheduleSave} />
          </label>
        </div>
      {/if}
    </div>
  {/each}
</div>

<button class="add-custom" onclick={addCustomReminder}>+ Custom reminder</button>

<style>
  h2 {
    margin-top: 0;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .card {
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 10px;
    overflow: hidden;
  }

  @media (prefers-color-scheme: dark) {
    .card {
      border-color: rgba(255, 255, 255, 0.15);
    }
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 0.9rem;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .emoji {
    font-size: 1.1rem;
  }

  .title {
    font-weight: 600;
    font-size: 0.9rem;
  }

  .header-actions {
    display: flex;
    gap: 0.75rem;
  }

  .link {
    background: none;
    border: none;
    color: #3478f6;
    font-size: 0.82rem;
    cursor: pointer;
    padding: 0;
  }

  .link.danger {
    color: #e0432b;
  }

  .card-body {
    padding: 0.75rem 0.9rem 1rem;
    border-top: 1px solid rgba(0, 0, 0, 0.08);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  @media (prefers-color-scheme: dark) {
    .card-body {
      border-top-color: rgba(255, 255, 255, 0.1);
    }
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .field span {
    font-size: 0.78rem;
    font-weight: 600;
    opacity: 0.75;
  }

  .field input[type="text"],
  .field input[type="number"],
  .field input[type="time"] {
    padding: 0.35rem 0.5rem;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.15);
    font-size: 0.85rem;
  }

  .field-row {
    display: flex;
    gap: 1rem;
  }
  .field-row .field {
    flex: 1;
  }

  .days {
    display: flex;
    gap: 0.3rem;
  }

  .day-pill {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.15);
    background: none;
    font-size: 0.75rem;
    cursor: pointer;
    color: inherit;
  }

  .day-pill.active {
    background: #3478f6;
    color: white;
    border-color: #3478f6;
  }

  .line-row {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  .line-row input {
    flex: 1;
  }

  .icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: #e0432b;
    font-size: 0.8rem;
  }

  .add-custom {
    margin-top: 1rem;
    background: none;
    border: 1px dashed rgba(0, 0, 0, 0.25);
    border-radius: 8px;
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 0.85rem;
    color: inherit;
  }
</style>
