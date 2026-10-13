<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { SpritePlayer, type SpritePackManifest } from "./SpritePlayer";
  import type { SceneState } from "./director";

  interface Props {
    sceneState: SceneState;
    manifest: SpritePackManifest;
    /** Absolute filesystem path to the pack's own directory (PackInfo.dir
     * from the Rust side), converted here into an asset:// URL the
     * webview is actually allowed to fetch from. */
    packDir: string;
    scale?: number;
    mirrored?: boolean;
    /** Caps frames fetched/decoded per state — pass a small number (e.g.
     * 1) for lightweight preview contexts like a Settings gallery
     * thumbnail. Undefined/null plays the full state normally. */
    maxFramesPerState?: number | null;
  }

  let {
    sceneState,
    manifest,
    packDir,
    scale = 1,
    mirrored = false,
    maxFramesPerState = null,
  }: Props = $props();

  let canvasEl: HTMLCanvasElement | undefined = $state();
  let player: SpritePlayer | undefined;

  // Real aspect ratio when the pack's manifest declares one (make_pack.py
  // writes widthPx), corrected again once actual frames decode (see
  // setOnFrameSize below) — this also covers older packs with no
  // widthPx, which start at a square guess until the real size arrives.
  let frameSizeOverride = $state<{ w: number; h: number } | null>(null);
  let aspect = $derived(
    frameSizeOverride
      ? frameSizeOverride.w / frameSizeOverride.h
      : manifest.widthPx
        ? manifest.widthPx / manifest.heightPx
        : 1,
  );
  // Packs are authored at whatever height their source video gave them
  // (make_pack.py defaults to 360-420px), but the built-in SVG characters
  // are ~160px. Rendering a pack at its native height makes it tower over
  // them and, more importantly, overlap the bubble's buttons. Normalise
  // every pack to the same on-screen height so the size slider means the
  // same thing whichever character is active.
  const TARGET_HEIGHT_PX = 168;

  let displayHeight = $derived(TARGET_HEIGHT_PX);
  let displayWidth = $derived(Math.round(displayHeight * aspect));

  let lastAppliedState: SceneState | undefined;

  onMount(() => {
    if (!canvasEl) return;
    const baseUrl = convertFileSrc(packDir);

    player = new SpritePlayer(canvasEl, manifest, baseUrl);
    player.maxFramesPerState = maxFramesPerState;
    player.setOnFrameSize((w, h) => {
      frameSizeOverride = { w, h };
    });
    player.start();
    // Only call setState here — the $effect below also watches sceneState
    // and used to call setState independently on its first run too, so
    // every mount fired setState() twice in quick succession. The second
    // call's closeState(this.current) could close bitmaps the first call
    // was still in the middle of loading, leaving current empty/closed
    // and nothing ever drawn — silently, with no error. Marking the
    // initial state as already-applied here keeps the effect from firing
    // again until sceneState actually changes.
    lastAppliedState = sceneState;
    player.setState(sceneState);
  });

  onDestroy(() => {
    player?.destroy();
  });

  // Keep the canvas's backing bitmap size in sync whenever the real
  // aspect ratio arrives/changes (decoded at 2x for a crisper draw than
  // the CSS display size).
  $effect(() => {
    if (canvasEl) {
      canvasEl.width = displayWidth * 2;
      canvasEl.height = displayHeight * 2;
    }
  });

  $effect(() => {
    if (player && sceneState !== lastAppliedState) {
      lastAppliedState = sceneState;
      player.setState(sceneState);
    }
  });
</script>

<div
  class="frames-character state-{sceneState}"
  class:mirrored
  style="--scale: {scale}; --frame-w: {displayWidth}px; --frame-h: {displayHeight}px;"
>
  <canvas
    bind:this={canvasEl}
    style="width: {displayWidth}px; height: {displayHeight}px;"
  ></canvas>
</div>

<style>
  .frames-character {
    --scale: 1;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    transform-origin: bottom center;
    /* The canvas is scaled with a transform, which layout ignores — so
       without these the wrapper stays at 1x and a scaled-up character
       grows straight over the bubble's buttons. Mirror the scale here in
       real layout units to reserve the room it actually occupies. */
    width: calc(var(--frame-w, 0px) * var(--scale));
    height: calc(var(--frame-h, 0px) * var(--scale));
  }

  canvas {
    display: block;
    transform: scale(var(--scale));
    transform-origin: bottom center;
    image-rendering: -webkit-optimize-contrast;
  }

  .frames-character.mirrored canvas {
    transform: scale(var(--scale)) scaleX(-1);
  }
</style>
