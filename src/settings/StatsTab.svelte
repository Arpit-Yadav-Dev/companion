<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { StatsSnapshot } from "../shared/types";

  let snapshot = $state<StatsSnapshot | null>(null);
  let goalInput = $state(8);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  onMount(async () => {
    snapshot = await invoke<StatsSnapshot>("get_stats_snapshot");
    if (snapshot) goalInput = snapshot.today.goal;
  });

  function scheduleSaveGoal() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      await invoke("save_daily_goal", { goal: goalInput });
      snapshot = await invoke<StatsSnapshot>("get_stats_snapshot");
    }, 300);
  }

  const CHART_HEIGHT = 100;
  const CHART_BAR_WIDTH = 28;
  const CHART_GAP = 14;

  function dayLabel(dateStr: string): string {
    const d = new Date(dateStr + "T00:00:00");
    return d.toLocaleDateString(undefined, { weekday: "short" }).slice(0, 2);
  }
</script>

<h2>Stats</h2>

{#if snapshot}
  <div class="today-row">
    <div class="stat-tile">
      <span class="stat-value">{snapshot.today.glasses} / {snapshot.today.goal}</span>
      <span class="stat-label">Glasses today</span>
    </div>
    <label class="field goal-field">
      <span>Daily goal</span>
      <input
        type="number"
        min="1"
        bind:value={goalInput}
        oninput={scheduleSaveGoal}
      />
    </label>
  </div>

  <h3>Last 7 days</h3>
  <svg
    class="chart"
    viewBox="0 0 {snapshot.lastSevenDays.length * (CHART_BAR_WIDTH + CHART_GAP)} {CHART_HEIGHT + 24}"
    width={snapshot.lastSevenDays.length * (CHART_BAR_WIDTH + CHART_GAP)}
    height={CHART_HEIGHT + 24}
  >
    {#each snapshot.lastSevenDays as day, i (day.date)}
      {@const goal = day.goal || 1}
      {@const ratio = Math.min(1, day.glasses / goal)}
      {@const barHeight = Math.max(2, ratio * CHART_HEIGHT)}
      {@const x = i * (CHART_BAR_WIDTH + CHART_GAP)}
      <rect
        x={x}
        y={CHART_HEIGHT - barHeight}
        width={CHART_BAR_WIDTH}
        height={barHeight}
        rx="4"
        fill={ratio >= 1 ? "#3478f6" : "#b9d4ff"}
      />
      <text x={x + CHART_BAR_WIDTH / 2} y={CHART_HEIGHT + 16} text-anchor="middle" class="axis-label">
        {dayLabel(day.date)}
      </text>
    {/each}
  </svg>

  <div class="streak-row">
    <div class="stat-tile">
      <span class="stat-value">{snapshot.currentStreak}</span>
      <span class="stat-label">Current streak</span>
    </div>
    <div class="stat-tile">
      <span class="stat-value">{snapshot.bestStreak}</span>
      <span class="stat-label">Best streak</span>
    </div>
  </div>

  <h3>Today's responses</h3>
  <div class="response-row">
    <span class="response yes">✓ {snapshot.today.yesCount} yes</span>
    <span class="response later">⏱ {snapshot.today.laterCount} later</span>
    <span class="response missed">— {snapshot.today.missedCount} missed</span>
  </div>
{/if}

<style>
  h2 {
    margin-top: 0;
  }
  h3 {
    font-size: 0.9rem;
    margin-top: 1.5rem;
    margin-bottom: 0.5rem;
  }

  .today-row {
    display: flex;
    align-items: flex-end;
    gap: 2rem;
  }

  .stat-tile {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    font-size: 1.6rem;
    font-weight: 700;
  }

  .stat-label {
    font-size: 0.78rem;
    opacity: 0.65;
  }

  .goal-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .goal-field span {
    font-size: 0.78rem;
    font-weight: 600;
    opacity: 0.75;
  }
  .goal-field input {
    width: 70px;
    padding: 0.3rem 0.5rem;
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.15);
  }

  .chart {
    display: block;
  }

  .axis-label {
    font-size: 9px;
    fill: currentColor;
    opacity: 0.6;
  }

  .streak-row {
    display: flex;
    gap: 2rem;
    margin-top: 1rem;
  }

  .response-row {
    display: flex;
    gap: 1.25rem;
    font-size: 0.85rem;
  }

  .response.yes {
    color: #2a9d5c;
  }
  .response.later {
    color: #c98a1a;
  }
  .response.missed {
    opacity: 0.6;
  }
</style>
