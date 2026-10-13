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

  const CHART_HEIGHT = 90;
  const CHART_BAR_WIDTH = 30;
  const CHART_GAP = 16;

  function dayLabel(dateStr: string): string {
    const d = new Date(dateStr + "T00:00:00");
    return d.toLocaleDateString(undefined, { weekday: "short" }).slice(0, 2);
  }
</script>

<h1 class="panel-title">Stats</h1>
<p class="panel-subtitle">Today's progress, this week's trend, and your streak.</p>

{#if snapshot}
  <section class="hero-row">
    <div class="card hero-tile">
      <span class="hero-value"
        >{snapshot.today.glasses}<span class="hero-of">/{snapshot.today.goal}</span></span
      >
      <span class="hero-label">Glasses today</span>
      <!-- The day's progress is the one number people come here for, so
           show it as a bar too rather than leaving them to divide. -->
      <span class="hero-progress" aria-hidden="true">
        <span
          class="hero-progress-fill"
          class:complete={snapshot.today.glasses >= snapshot.today.goal}
          style="width: {Math.min(100, (snapshot.today.glasses / Math.max(1, snapshot.today.goal)) * 100)}%"
        ></span>
      </span>
    </div>
    <div class="card hero-tile">
      <span class="hero-value">{snapshot.currentStreak}<span class="hero-unit">d</span></span>
      <span class="hero-label">Current streak</span>
    </div>
    <div class="card hero-tile">
      <span class="hero-value">{snapshot.bestStreak}<span class="hero-unit">d</span></span>
      <span class="hero-label">Best streak</span>
    </div>
  </section>

  <section class="section card pad">
    <div class="section-header">
      <h2 class="section-title">Last 7 days</h2>
      <label class="goal-field">
        <span class="field-label">Daily goal</span>
        <input type="number" min="1" bind:value={goalInput} oninput={scheduleSaveGoal} />
      </label>
    </div>

    <svg
      class="chart"
      viewBox="0 0 {snapshot.lastSevenDays.length * (CHART_BAR_WIDTH + CHART_GAP)} {CHART_HEIGHT + 22}"
      width={snapshot.lastSevenDays.length * (CHART_BAR_WIDTH + CHART_GAP)}
      height={CHART_HEIGHT + 22}
    >
      <!-- The goal line turns each bar from a raw count into a pass/fail
           at a glance; without it the bars only compare days to each
           other, which isn't what the goal is for. -->
      <line
        class="goal-line"
        x1="0"
        y1="0.5"
        x2={snapshot.lastSevenDays.length * (CHART_BAR_WIDTH + CHART_GAP) - CHART_GAP}
        y2="0.5"
      />
      {#each snapshot.lastSevenDays as day, i (day.date)}
        {@const goal = day.goal || 1}
        {@const ratio = Math.min(1, day.glasses / goal)}
        {@const barHeight = Math.max(3, ratio * CHART_HEIGHT)}
        {@const x = i * (CHART_BAR_WIDTH + CHART_GAP)}
        <rect
          x={x}
          y={CHART_HEIGHT - barHeight}
          width={CHART_BAR_WIDTH}
          height={barHeight}
          rx="6"
          class="bar"
          class:bar-complete={ratio >= 1}
        />
        <text
          x={x + CHART_BAR_WIDTH / 2}
          y={CHART_HEIGHT + 16}
          text-anchor="middle"
          class="axis-label"
        >
          {dayLabel(day.date)}
        </text>
      {/each}
    </svg>
  </section>

  <section class="section card pad">
    <h2 class="section-title">Today's responses</h2>
    <div class="response-row">
      <span class="response-pill yes">✓ {snapshot.today.yesCount} yes</span>
      <span class="response-pill later">⏱ {snapshot.today.laterCount} later</span>
      <span class="response-pill missed">— {snapshot.today.missedCount} missed</span>
    </div>
  </section>
{/if}

<style>
  .hero-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }

  .hero-tile {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-4) var(--space-4);
  }

  .hero-value {
    font-family: var(--font-display);
    font-size: 2rem;
    font-weight: 750;
    color: var(--text);
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }

  .hero-of,
  .hero-unit {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-faint);
  }

  .hero-label {
    font-size: 0.78rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .hero-progress {
    display: block;
    width: 100%;
    height: 5px;
    margin-top: var(--space-2);
    border-radius: var(--radius-pill);
    background: var(--surface-sunken);
    overflow: hidden;
  }

  .hero-progress-fill {
    display: block;
    height: 100%;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    transition: width 0.3s ease;
  }
  .hero-progress-fill.complete {
    background: var(--success);
  }

  .section {
    margin-bottom: var(--space-4);
  }
  .section.pad {
    padding: var(--space-4) var(--space-5);
  }

  .section-header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .section-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text);
  }

  .goal-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
  .goal-field input {
    width: 70px;
    text-align: center;
  }

  .chart {
    display: block;
    overflow: visible;
  }

  .bar {
    fill: var(--accent-soft);
  }
  .bar-complete {
    fill: var(--accent);
  }

  .goal-line {
    stroke: var(--border-strong);
    stroke-width: 1;
    stroke-dasharray: 3 3;
  }

  .axis-label {
    font-size: 10px;
    font-weight: 600;
    fill: var(--text-faint);
  }

  .response-row {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .response-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0.4rem 0.8rem;
    border-radius: 999px;
    font-size: 0.82rem;
    font-weight: 650;
  }

  .response-pill.yes {
    color: var(--success);
    background: color-mix(in srgb, var(--success) 14%, transparent);
  }
  .response-pill.later {
    color: var(--warning);
    background: color-mix(in srgb, var(--warning) 14%, transparent);
  }
  .response-pill.missed {
    color: var(--text-muted);
    background: var(--surface-hover);
  }
</style>
