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

  let responseParts = $derived([
    { key: "yes", label: "answered", count: snapshot?.today.yesCount ?? 0 },
    { key: "later", label: "deferred", count: snapshot?.today.laterCount ?? 0 },
    { key: "missed", label: "missed", count: snapshot?.today.missedCount ?? 0 },
  ]);
  let totalResponses = $derived(responseParts.reduce((sum, p) => sum + p.count, 0));

  function adjustGoal(delta: number) {
    goalInput = Math.min(20, Math.max(1, goalInput + delta));
    scheduleSaveGoal();
  }

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
      <div class="goal-field">
        <span class="field-label">Daily goal</span>
        <!-- Stepper rather than a bare number box: the goal moves by one
             glass at a time, and the dashed line on the chart below is
             what it actually controls. -->
        <div class="stepper">
          <button onclick={() => adjustGoal(-1)} disabled={goalInput <= 1} aria-label="Lower goal"
            >−</button
          >
          <span class="stepper-value">{goalInput}</span>
          <button onclick={() => adjustGoal(1)} disabled={goalInput >= 20} aria-label="Raise goal"
            >+</button
          >
        </div>
      </div>
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
    {#if totalResponses > 0}
      <!-- One stacked bar rather than three separate counts: how the day
           split between answering, deferring and ignoring is the point,
           and proportions are what show that. -->
      <div class="response-bar" aria-hidden="true">
        {#each responseParts as part (part.key)}
          {#if part.count > 0}
            <span
              class="response-part {part.key}"
              style="width: {(part.count / totalResponses) * 100}%"
            ></span>
          {/if}
        {/each}
      </div>
      <div class="response-legend">
        {#each responseParts as part (part.key)}
          <span class="legend-item" class:zero={part.count === 0}>
            <span class="legend-dot {part.key}"></span>
            {part.count}
            {part.label}
          </span>
        {/each}
      </div>
    {:else}
      <p class="empty-note">No reminders yet today.</p>
    {/if}
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

  .stepper {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }

  .stepper button {
    width: 26px;
    height: 24px;
    border: none;
    border-radius: 5px;
    background: none;
    color: var(--text-muted);
    font-size: 0.95rem;
    font-family: var(--font-body);
    cursor: pointer;
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
  .stepper button:hover:not(:disabled) {
    background: var(--surface-hover);
    color: var(--text);
  }
  .stepper button:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .stepper-value {
    min-width: 24px;
    text-align: center;
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text);
    font-variant-numeric: tabular-nums;
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

  .response-bar {
    display: flex;
    width: 100%;
    height: 10px;
    border-radius: var(--radius-pill);
    overflow: hidden;
    background: var(--surface-sunken);
  }

  .response-part {
    display: block;
    height: 100%;
    transition: width 0.3s ease;
  }
  .response-part.yes {
    background: var(--success);
  }
  .response-part.later {
    background: var(--warning);
  }
  .response-part.missed {
    background: var(--border-strong);
  }

  .response-legend {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
    margin-top: var(--space-3);
  }

  .legend-item {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }
  .legend-item.zero {
    color: var(--text-faint);
  }

  .legend-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .legend-dot.yes {
    background: var(--success);
  }
  .legend-dot.later {
    background: var(--warning);
  }
  .legend-dot.missed {
    background: var(--border-strong);
  }

  .empty-note {
    margin: 0;
    font-size: 0.82rem;
    color: var(--text-muted);
  }
</style>
