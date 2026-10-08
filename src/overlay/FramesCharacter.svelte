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
  }

  let { sceneState, manifest, packDir, scale = 1, mirrored = false }: Props = $props();

  let canvasEl: HTMLCanvasElement | undefined = $state();
  let player: SpritePlayer | undefined;

  // Placeholder aspect until the first real frame paints — the canvas
  // itself resizes via CSS regardless once content is drawn.
  let displayWidth = $derived(Math.round(manifest.heightPx * 0.8));
  let displayHeight = $derived(manifest.heightPx);

  onMount(() => {
    if (!canvasEl) return;
    const baseUrl = convertFileSrc(packDir);
    // Decode at a modest fixed resolution — the canvas is styled down to
    // its display size in CSS regardless, and keeping the backing bitmap
    // smaller than the source frames avoids holding huge GPU textures for
    // a 420px-tall pack scaled down in a settings preview.
    canvasEl.width = displayWidth * 2;
    canvasEl.height = displayHeight * 2;

    player = new SpritePlayer(canvasEl, manifest, baseUrl);
    player.start();
    player.setState(sceneState);
  });

  onDestroy(() => {
    player?.destroy();
  });

  let lastAppliedState: SceneState | undefined;
  $effect(() => {
    if (player && sceneState !== lastAppliedState) {
      lastAppliedState = sceneState;
      player.setState(sceneState);
    }
  });
</script>

<div class="frames-character state-{sceneState}" class:mirrored style="--scale: {scale}">
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
