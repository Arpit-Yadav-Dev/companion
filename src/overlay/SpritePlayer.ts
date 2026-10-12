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
  frameCount?: number;
}

export interface SpritePackManifest {
  id: string;
  name: string;
  renderer: "svg" | "frames";
  fps: number;
  heightPx: number;
  /** Real frame width in px, written by make_pack.py. Falls back to a
   * guess (via onFrameSize) only for packs made before this field existed. */
  widthPx?: number;
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

  /** Real decoded frame size, set once the first bitmap of the first
   * loaded state is available. Packs made before `widthPx`/`heightPx`
   * were written per-frame still get the true size this way, since it's
   * read directly off the decoded image rather than trusted from JSON. */
  private frameWidth = 0;
  private frameHeight = 0;
  private onFrameSize: ((width: number, height: number) => void) | null = null;

  constructor(canvas: HTMLCanvasElement, manifest: SpritePackManifest, baseUrl: string) {
    this.canvas = canvas;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("SpritePlayer: 2d canvas context unavailable");
    this.ctx = ctx;
    this.manifest = manifest;
    this.baseUrl = baseUrl;
  }

  /** Called (at most once) as soon as the real frame dimensions are known,
   * so the caller can size the canvas/display box correctly instead of
   * guessing an aspect ratio before anything has decoded. */
  setOnFrameSize(cb: (width: number, height: number) => void) {
    this.onFrameSize = cb;
    if (this.frameWidth && this.frameHeight) cb(this.frameWidth, this.frameHeight);
  }

  /** Counts frames in a state's directory by probing sequential URLs until
   * one fails to load. Only used as a fallback for packs made before
   * pack.json carried a per-state `frameCount` (make_pack.py now writes
   * it, so this slow path shouldn't normally run). */
  private async countFrames(dir: string): Promise<number> {
    let count = 0;
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

  /** Caps how many frames of any state get fetched/decoded. Set this for
   * lightweight contexts (e.g. a small Settings gallery thumbnail) where
   * a single still frame is enough and decoding a full 300-frame loop is
   * wasted work — doubly so when the same pack might be mounted more
   * than once at a time (its own gallery card plus a "currently active"
   * preview), which otherwise doubles the simultaneous fetch load. */
  maxFramesPerState: number | null = null;

  private async loadState(stateName: string): Promise<LoadedState | null> {
    const config = this.manifest.states[stateName];
    if (!config) return null;

    let frameCount = config.frameCount ?? (await this.countFrames(config.dir));
    if (this.maxFramesPerState) frameCount = Math.min(frameCount, this.maxFramesPerState);
    if (frameCount === 0) return null;

    // Fetch + decode every frame in parallel rather than one at a time —
    // sequential awaits on a few hundred frames can take seconds, long
    // enough that a fixed-duration entrance animation (CSS slide-in,
    // ~1.2s) finishes before any frame has even arrived, so the character
    // appears to pop into place instead of walking in.
    //
    // Using allSettled (not all) matters when more than one SpritePlayer
    // is alive at once (e.g. the Settings window renders the same active
    // pack twice — once in the "currently active" hero, once in its own
    // gallery card) — that much simultaneous fetch/decode load can make a
    // handful of individual frames fail (empty buffer), and Promise.all
    // would reject the *entire* state over one bad frame, leaving nothing
    // drawn at all. Settling individually keeps every frame that did
    // decode and simply skips the rare one that didn't.
    const settled = await Promise.allSettled(
      Array.from({ length: frameCount }, async (_, i) => {
        const response = await fetch(frameUrl(this.baseUrl, config.dir, i));
        const blob = await response.blob();
        return createImageBitmap(blob);
      }),
    );
    const bitmaps = settled
      .filter((r): r is PromiseFulfilledResult<ImageBitmap> => r.status === "fulfilled")
      .map((r) => r.value);
    if (bitmaps.length === 0) return null;

    if (!this.frameWidth && bitmaps.length > 0) {
      this.frameWidth = bitmaps[0].width;
      this.frameHeight = bitmaps[0].height;
      this.onFrameSize?.(this.frameWidth, this.frameHeight);
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
    // bitmap.width throws InvalidStateError once close() has been called
    // on it — setState() can close the previously-current state's bitmaps
    // asynchronously while this rAF-driven draw() is already in flight for
    // the same frame, so a bitmap reference captured a tick ago can be
    // stale by the time we get here.
    if (!bitmap || bitmap.width === 0) return;

    // Scale to fit the canvas while preserving the bitmap's real aspect
    // ratio — stretching to whatever width/height the canvas happens to
    // have (ignoring the source aspect) is what caused distorted/squished
    // frames when the canvas was sized from a guessed ratio.
    const scale = Math.min(width / bitmap.width, height / bitmap.height);
    const drawWidth = bitmap.width * scale;
    const drawHeight = bitmap.height * scale;
    const dx = (width - drawWidth) / 2;
    const dy = height - drawHeight; // anchor to the bottom (feet stay planted)
    try {
      this.ctx.drawImage(bitmap, dx, dy, drawWidth, drawHeight);
    } catch {
      // Bitmap was closed between the width check above and this call
      // (can happen under rapid state switches) — skip this frame rather
      // than crash; the next rAF tick draws the new state correctly.
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
