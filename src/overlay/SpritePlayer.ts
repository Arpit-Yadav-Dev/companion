// Canvas frame-sequence player for `frames`-renderer packs (the
// pre-rendered/3D look). Decodes frames for only the current and next
// state — never a whole pack at once, since e.g. 300 frames at 420px
// could be hundreds of MB of decoded bitmaps — and runs a fixed-fps
// accumulator loop that's fully stopped (not just paused) whenever the
// overlay hides, so there's zero ongoing CPU/GPU cost while idle.

export interface SpriteStateConfig {
  dir: string;
  loop: boolean;
  move?: "enter" | "exit";
}

export interface SpritePackManifest {
  id: string;
  name: string;
  renderer: "svg" | "frames";
  fps: number;
  heightPx: number;
  anchor: { x: number; y: number };
  bubbleOffset: { x: number; y: number };
  states: Record<string, SpriteStateConfig>;
  props: Record<string, string>;
  mirrorForDirection: boolean;
  stridePx?: number;
}

interface LoadedState {
  dirKey: string;
  bitmaps: ImageBitmap[];
}

/**
 * Builds the frame file URL for a given state's directory and frame
 * index. Packs store frames as zero-padded sequential WebP files, e.g.
 * `walk/0001.webp`. The `baseUrl` is the pack's own directory, exposed to
 * the frontend as an `asset://` (or plain `http://`-in-dev) URL by the
 * Rust side's pack listing.
 */
function frameUrl(baseUrl: string, dir: string, frameIndex: number): string {
  const padded = String(frameIndex + 1).padStart(4, "0");
  return `${baseUrl}/${dir}/${padded}.webp`;
}

export class SpritePlayer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private manifest: SpritePackManifest;
  private baseUrl: string;

  private current: LoadedState | null = null;
  private next: LoadedState | null = null;

  private frameIndex = 0;
  private accumulatorMs = 0;
  private lastTimestamp = 0;
  private rafHandle: number | null = null;
  private running = false;

  private onLoopComplete: (() => void) | null = null;

  constructor(canvas: HTMLCanvasElement, manifest: SpritePackManifest, baseUrl: string) {
    this.canvas = canvas;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("SpritePlayer: 2d canvas context unavailable");
    this.ctx = ctx;
    this.manifest = manifest;
    this.baseUrl = baseUrl;
  }

  /** Counts frames in a state's directory by probing sequential URLs until
   * one fails to load. Packs are small (a few seconds of animation), so a
   * linear probe is simpler and cheap enough — avoids requiring a
   * separate per-state frame-count field in pack.json. */
  private async countFrames(dir: string): Promise<number> {
    let count = 0;
    // Hard cap matches the brief's "a few hundred frames" ballpark; a
    // well-formed pack will never approach this, it just bounds a
    // pathological/corrupt pack's probe loop.
    const MAX_FRAMES = 1000;
    while (count < MAX_FRAMES) {
      const ok = await new Promise<boolean>((resolve) => {
        const img = new Image();
        img.onload = () => resolve(true);
        img.onerror = () => resolve(false);
        img.src = frameUrl(this.baseUrl, dir, count);
      });
      if (!ok) break;
      count++;
    }
    return count;
  }

  private async loadState(stateName: string): Promise<LoadedState | null> {
    const config = this.manifest.states[stateName];
    if (!config) return null;

    const frameCount = await this.countFrames(config.dir);
    if (frameCount === 0) return null;

    const bitmaps: ImageBitmap[] = [];
    for (let i = 0; i < frameCount; i++) {
      const response = await fetch(frameUrl(this.baseUrl, config.dir, i));
      const blob = await response.blob();
      bitmaps.push(await createImageBitmap(blob));
    }

    return { dirKey: config.dir, bitmaps };
  }

  private closeState(state: LoadedState | null) {
    if (!state) return;
    for (const bitmap of state.bitmaps) {
      bitmap.close();
    }
  }

  /** Switches the active state, decoding the new one only if it isn't
   * already the pre-fetched "next" state, and closing whatever was
   * loaded before so memory never holds more than ~2 states' bitmaps. */
  async setState(stateName: string, onLoopComplete?: () => void) {
    this.onLoopComplete = onLoopComplete ?? null;
    this.frameIndex = 0;
    this.accumulatorMs = 0;

    if (this.next && this.next.dirKey === this.manifest.states[stateName]?.dir) {
      this.closeState(this.current);
      this.current = this.next;
      this.next = null;
    } else {
      this.closeState(this.current);
      this.current = await this.loadState(stateName);
    }

    this.draw();
  }

  /** Pre-decodes a likely-next state's frames in the background, so the
   * transition into it (e.g. ask -> happy on a YES click) has no decode
   * stall. Safe to call speculatively; a wrong guess just gets dropped
   * next `setState` call. */
  async prefetchState(stateName: string) {
    const config = this.manifest.states[stateName];
    if (!config) return;
    if (this.next?.dirKey === config.dir || this.current?.dirKey === config.dir) return;
    this.next = await this.loadState(stateName);
  }

  start() {
    if (this.running) return;
    this.running = true;
    this.lastTimestamp = performance.now();
    const loop = (timestamp: number) => {
      if (!this.running) return;
      const deltaMs = timestamp - this.lastTimestamp;
      this.lastTimestamp = timestamp;
      this.accumulatorMs += deltaMs;

      const frameDurationMs = 1000 / this.manifest.fps;
      let advanced = false;
      while (this.accumulatorMs >= frameDurationMs) {
        this.accumulatorMs -= frameDurationMs;
        this.advanceFrame();
        advanced = true;
      }
      if (advanced) this.draw();

      this.rafHandle = requestAnimationFrame(loop);
    };
    this.rafHandle = requestAnimationFrame(loop);
  }

  /** Fully stops the rAF loop — not a pause flag, an actual cancelled
   * callback — so a hidden overlay costs nothing. */
  stop() {
    this.running = false;
    if (this.rafHandle !== null) {
      cancelAnimationFrame(this.rafHandle);
      this.rafHandle = null;
    }
  }

  private advanceFrame() {
    if (!this.current || this.current.bitmaps.length === 0) return;
    const config = this.manifest.states[this.stateNameForCurrent()];
    const isLoop = config?.loop ?? false;

    this.frameIndex++;
    if (this.frameIndex >= this.current.bitmaps.length) {
      if (isLoop) {
        this.frameIndex = 0;
      } else {
        this.frameIndex = this.current.bitmaps.length - 1;
        this.onLoopComplete?.();
      }
    }
  }

  private stateNameForCurrent(): string {
    for (const [name, config] of Object.entries(this.manifest.states)) {
      if (config.dir === this.current?.dirKey) return name;
    }
    return "";
  }

  private draw() {
    const bitmap = this.current?.bitmaps[this.frameIndex];
    const { width, height } = this.canvas;
    this.ctx.clearRect(0, 0, width, height);
    if (bitmap) {
      this.ctx.drawImage(bitmap, 0, 0, width, height);
    }
  }

  destroy() {
    this.stop();
    this.closeState(this.current);
    this.closeState(this.next);
    this.current = null;
    this.next = null;
  }
}
