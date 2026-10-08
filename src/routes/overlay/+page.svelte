<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import type { HitRect, ReminderShowPayload, ReminderChoice } from "../../shared/types";

  let visible = $state(false);
  let line = $state("");
  let yesLabel = $state("YES");
  let laterLabel = $state("Remind me later");
  let reminderId = $state("");

  let boxEl: HTMLDivElement | undefined = $state();
  let timeoutHandle: ReturnType<typeof setTimeout> | undefined;

  const TIMEOUT_MS = 25_000;

  function reportHitRegions() {
    if (!boxEl || !visible) {
      invoke("set_hit_regions", { rects: [] as HitRect[] });
      return;
    }
    const rect = boxEl.getBoundingClientRect();
    const rects: HitRect[] = [
      { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
    ];
    invoke("set_hit_regions", { rects });
  }

  async function resolve(choice: ReminderChoice) {
    if (timeoutHandle) clearTimeout(timeoutHandle);
    visible = false;
    await reportHitRegions();
    await invoke("reminder_response", {
      payload: { reminderId, choice },
    });
  }

  onMount(() => {
    const webview = getCurrentWebviewWindow();

    const unlistenPromise = webview.listen<ReminderShowPayload>(
      "reminder://show",
      async (event) => {
        const payload = event.payload;
        reminderId = payload.reminderId;
        yesLabel = payload.yesLabel;
        laterLabel = payload.laterLabel;
        line = payload.lines[Math.floor(Math.random() * payload.lines.length)] ?? "";
        visible = true;

        await new Promise((r) => requestAnimationFrame(r));
        reportHitRegions();
        await invoke("overlay_ready");

        if (timeoutHandle) clearTimeout(timeoutHandle);
        timeoutHandle = setTimeout(() => resolve("timeout"), TIMEOUT_MS);
      },
    );

    const resizeObserver = new ResizeObserver(() => reportHitRegions());
    if (boxEl) resizeObserver.observe(boxEl);

    function onKeydown(e: KeyboardEvent) {
      if (!visible) return;
      if (e.key === "y" || e.key === "Y") resolve("yes");
      else if (e.key === "l" || e.key === "L") resolve("later");
      else if (e.key === "Escape") resolve("later");
    }
    window.addEventListener("keydown", onKeydown);

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
      resizeObserver.disconnect();
      window.removeEventListener("keydown", onKeydown);
      if (timeoutHandle) clearTimeout(timeoutHandle);
    };
  });
</script>

<div class="stage">
  {#if visible}
    <div class="placeholder-box" bind:this={boxEl}>
      <p class="line">{line}</p>
      <div class="buttons">
        <button class="pill yes" onclick={() => resolve("yes")}>{yesLabel}</button>
        <button class="pill later" onclick={() => resolve("later")}>{laterLabel}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    background: transparent;
    height: 100%;
    overflow: hidden;
  }

  .stage {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: flex-end;
    justify-content: flex-start;
    padding: 2rem 0 3rem 3rem;
    pointer-events: none;
  }

  .placeholder-box {
    pointer-events: auto;
    background: white;
    border-radius: 20px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.25);
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    min-width: 260px;
    font-family:
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      sans-serif;
  }

  .line {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
    color: #222;
    text-align: center;
  }

  .buttons {
    display: flex;
    gap: 0.75rem;
  }

  .pill {
    border: none;
    border-radius: 999px;
    padding: 0.6rem 1.4rem;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    background: white;
    color: #222;
    box-shadow:
      0 2px 6px rgba(0, 0, 0, 0.15),
      inset 0 0 0 1.5px #ddd;
    transition:
      transform 0.1s ease,
      box-shadow 0.1s ease;
  }

  .pill:hover {
    transform: translateY(-2px);
    box-shadow:
      0 4px 10px rgba(0, 0, 0, 0.2),
      inset 0 0 0 1.5px #ccc;
  }

  .pill:active {
    transform: translateY(0) scale(0.96);
  }

  .pill.yes {
    background: #ffe14d;
  }
</style>
