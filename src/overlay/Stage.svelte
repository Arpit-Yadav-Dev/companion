<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import type { HitRect, ReminderShowPayload } from "../shared/types";
  import { Director, type SceneState, type EntranceSide } from "./director";
  import SvgCharacter from "./SvgCharacter.svelte";
  import { svgCharacterFor } from "./characterRegistry";
  import FramesCharacter from "./FramesCharacter.svelte";
  import Bubble from "./Bubble.svelte";
  import Confetti from "./Confetti.svelte";
  import Ribbons from "./Ribbons.svelte";
  import type { CharacterPrefs, GeneralPrefs, PackInfo } from "../shared/types";

  let sceneState = $state<SceneState>("hidden");
  let line = $state("");
  let buttonsVisible = $state(false);
  let entranceSide = $state<EntranceSide>("left");
  let yesLabel = $state("YES");
  let laterLabel = $state("Remind me later");
  let showConfetti = $state(false);
  let showRibbons = $state(false);
  let activeProp = $state<string | null>(null);

  let characterEl: HTMLDivElement | undefined = $state();
  let bubbleEl: HTMLDivElement | undefined = $state();

  let characterPrefs = $state<CharacterPrefs | null>(null);
  let generalPrefs = $state<GeneralPrefs | null>(null);
  let activeFramesPack = $state<PackInfo | null>(null);
  let ActiveCharacter = $derived(svgCharacterFor(characterPrefs?.activePackId));

  const director = new Director({
    onStateChange: (s) => (sceneState = s),
    onLineChange: (l) => (line = l),
    onButtonsVisibleChange: (v) => (buttonsVisible = v),
    onEntranceSideChange: (s) => (entranceSide = s),
    onPropChange: (p) => (activeProp = p),
    onGoalReached: () => {
      showConfetti = true;
      setTimeout(() => (showConfetti = false), 1600);
    },
    onYesCelebrate: () => {
      showRibbons = true;
      setTimeout(() => (showRibbons = false), 900);
    },
  });

  // --- Easter egg: clicking the character itself (not the buttons) 5
  // times within 1.5s triggers a special reaction. ---
  let clickTimestamps: number[] = [];
  let easterEggActive = $state(false);

  function onCharacterClick() {
    const now = Date.now();
    clickTimestamps = [...clickTimestamps, now].filter((t) => now - t < 1500);
    if (clickTimestamps.length >= 5) {
      clickTimestamps = [];
      easterEggActive = true;
      setTimeout(() => (easterEggActive = false), 1200);
    }
  }

  function reportHitRegions() {
    const rects: HitRect[] = [];
    if (sceneState !== "hidden") {
      if (characterEl) {
        const r = characterEl.getBoundingClientRect();
        rects.push({ x: r.x, y: r.y, width: r.width, height: r.height });
      }
      if (bubbleEl && buttonsVisible) {
        const r = bubbleEl.getBoundingClientRect();
        rects.push({ x: r.x, y: r.y, width: r.width, height: r.height });
      }
    }
    invoke("set_hit_regions", { rects });
  }

  function onYes() {
    director.choose("yes");
  }
  function onLater() {
    director.choose("later");
  }

  function onKeydown(e: KeyboardEvent) {
    if (!buttonsVisible) return;
    if (e.key === "y" || e.key === "Y") onYes();
    else if (e.key === "l" || e.key === "L") onLater();
    else if (e.key === "Escape") onLater();
  }

  let resizeObserver: ResizeObserver | undefined;

  onMount(() => {
    const webview = getCurrentWebviewWindow();

    const unlistenPromise = webview.listen<ReminderShowPayload>(
      "reminder://show",
      async (event) => {
        // Re-fetch prefs on every appearance (cheap, and the overlay is
        // hidden almost all the time) so Settings edits take effect
        // without needing a dedicated live-reload event or an overlay
        // restart.
        let packs: PackInfo[];
        [characterPrefs, generalPrefs, packs] = await Promise.all([
          invoke<CharacterPrefs>("get_character_prefs"),
          invoke<GeneralPrefs>("get_general_prefs"),
          invoke<PackInfo[]>("list_packs"),
        ]);
        activeFramesPack =
          packs.find(
            (p) => p.manifest.id === characterPrefs?.activePackId && p.manifest.renderer === "frames",
          ) ?? null;

        const prefersReducedMotionMedia = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        director.setReducedMotion(generalPrefs.reducedMotion || prefersReducedMotionMedia);
        director.setSoundPrefs({ enabled: generalPrefs.soundEnabled, volume: generalPrefs.soundVolume });
        director.setAnimationSpeedPercent(characterPrefs.animationSpeedPercent ?? 100);

        // Show the window first — requestAnimationFrame never fires on a
        // hidden window in WKWebView, so anything waiting on a frame
        // before this point would hang forever.
        await invoke("overlay_ready");
        yesLabel = event.payload.yesLabel;
        laterLabel = event.payload.laterLabel;
        director.playScene(event.payload, characterPrefs.entranceSide);
      },
    );

    resizeObserver = new ResizeObserver(() => reportHitRegions());
    window.addEventListener("keydown", onKeydown);

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
      window.removeEventListener("keydown", onKeydown);
      resizeObserver?.disconnect();
    };
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
  });

  $effect(() => {
    // Re-observe whenever the elements this effect reads change identity
    // (e.g. first render), and re-report whenever state/buttons change.
    if (characterEl) resizeObserver?.observe(characterEl);
    if (bubbleEl) resizeObserver?.observe(bubbleEl);
    sceneState;
    buttonsVisible;
    queueMicrotask(reportHitRegions);
  });
</script>

<div class="stage">
  {#if sceneState !== "hidden"}
    <div
      class="scene side-{entranceSide} pos-{(characterPrefs?.position ?? 'bottom_left').replace('_', '-')}"
      class:reduced-motion={generalPrefs?.reducedMotion}
      style="--walk-ms: {1200 / (Math.max(characterPrefs?.animationSpeedPercent ?? 100, 10) / 100)}ms; --anim-speed: {(characterPrefs?.animationSpeedPercent ?? 100) / 100}; --char-scale: {(characterPrefs?.sizePercent ?? 100) / 100};"
    >
      <div class="bubble-wrap" bind:this={bubbleEl}>
        {#if line}
          <Bubble
            text={line}
            {yesLabel}
            {laterLabel}
            showButtons={buttonsVisible}
            {onYes}
            onLater={onLater}
          />
        {/if}
      </div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="character-wrap state-{sceneState}"
        class:easter-egg={easterEggActive}
        bind:this={characterEl}
        onclick={onCharacterClick}
        role="button"
        tabindex="-1"
      >
        {#if showConfetti}
          <Confetti />
        {/if}
        {#if showRibbons}
          <Ribbons />
        {/if}
        {#if activeFramesPack}
          <FramesCharacter
            {sceneState}
            manifest={activeFramesPack.manifest}
            packDir={activeFramesPack.dir}
            mirrored={entranceSide === "right"}
            scale={(characterPrefs?.sizePercent ?? 100) / 100}
          />
        {:else if ActiveCharacter}
          <ActiveCharacter
            {sceneState}
            prop={activeProp}
            mirrored={entranceSide === "right"}
            theme={characterPrefs?.theme ?? "blue"}
            scale={(characterPrefs?.sizePercent ?? 100) / 100}
          />
        {:else}
          <SvgCharacter
            {sceneState}
            mirrored={entranceSide === "right"}
            theme={characterPrefs?.theme ?? "blue"}
            scale={(characterPrefs?.sizePercent ?? 100) / 100}
          />
        {/if}
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
    pointer-events: none;
  }

  .scene {
    position: absolute;
    bottom: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    /* Every character now reports its scaled size in real layout units
       (SVG characters size via width/height; FramesCharacter mirrors its
       canvas transform onto its wrapper), so flex reserves the right room
       on its own and the gap is just breathing space. This used to carry
       a large scale-dependent term to compensate for transform-scaled
       characters growing over the buttons — that belonged in the
       characters themselves, not here. */
    gap: 0.75rem;
  }

  .scene.pos-bottom-left {
    left: 4rem;
  }
  .scene.pos-bottom-right {
    right: 4rem;
  }
  .scene.pos-bottom-center {
    left: 50%;
    translate: -50% 0;
  }

  .bubble-wrap,
  .character-wrap {
    pointer-events: auto;
    display: flex;
    justify-content: center;
  }

  /* Walk-in/out: slide between the screen edge and the resting spot. The
     resting position is 0; walk_in starts off-screen and animates to 0,
     walk_out animates from 0 back off-screen. Mirror the character so it
     visually faces the direction it's walking. */
  .character-wrap {
    transform: translateX(0);
  }

  .side-left .character-wrap.state-walk_in {
    animation: slide-in-from-left var(--walk-ms, 1200ms) ease-out both;
  }
  .side-left .character-wrap.state-walk_out {
    animation: slide-out-to-left var(--walk-ms, 1200ms) ease-in both;
  }
  .side-right .character-wrap.state-walk_in {
    animation: slide-in-from-right var(--walk-ms, 1200ms) ease-out both;
  }
  .side-right .character-wrap.state-walk_out {
    animation: slide-out-to-right var(--walk-ms, 1200ms) ease-in both;
  }

  .side-right .character-wrap {
    transform: scaleX(-1);
  }

  @keyframes slide-in-from-left {
    from {
      transform: translateX(-60vw);
    }
    to {
      transform: translateX(0);
    }
  }
  @keyframes slide-out-to-left {
    from {
      transform: translateX(0);
    }
    to {
      transform: translateX(-60vw);
    }
  }
  @keyframes slide-in-from-right {
    from {
      transform: translateX(60vw) scaleX(-1);
    }
    to {
      transform: translateX(0) scaleX(-1);
    }
  }
  @keyframes slide-out-to-right {
    from {
      transform: translateX(0) scaleX(-1);
    }
    to {
      transform: translateX(60vw) scaleX(-1);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .character-wrap.state-walk_in,
    .character-wrap.state-walk_out {
      animation: none;
    }
    .scene {
      animation: crossfade 300ms ease both;
    }
  }

  /* Same override as the media query above, but driven by the app's own
     "Reduced motion" setting rather than (or in addition to) the OS
     preference — the director also shortens its walk-state timing to
     match (see REDUCED_WALK_MS in director.ts) so the crossfade has time
     to finish before the state moves on. */
  .scene.reduced-motion .character-wrap.state-walk_in,
  .scene.reduced-motion .character-wrap.state-walk_out {
    animation: none;
  }
  .scene.reduced-motion {
    animation: crossfade 220ms ease both;
  }

  @keyframes crossfade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .easter-egg {
    animation: easter-egg-spin 1.2s ease-in-out;
  }
  @keyframes easter-egg-spin {
    0% {
      transform: rotate(0deg) scale(1);
    }
    30% {
      transform: rotate(20deg) scale(1.15);
    }
    60% {
      transform: rotate(-15deg) scale(1.1);
    }
    100% {
      transform: rotate(0deg) scale(1);
    }
  }
  .side-right .character-wrap.easter-egg {
    /* Preserve the mirror while still applying the spin/scale. */
    animation-name: easter-egg-spin-mirrored;
  }
  @keyframes easter-egg-spin-mirrored {
    0% {
      transform: scaleX(-1) rotate(0deg) scale(1);
    }
    30% {
      transform: scaleX(-1) rotate(-20deg) scale(1.15);
    }
    60% {
      transform: scaleX(-1) rotate(15deg) scale(1.1);
    }
    100% {
      transform: scaleX(-1) rotate(0deg) scale(1);
    }
  }
</style>
