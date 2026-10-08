import { invoke } from "@tauri-apps/api/core";
import type { ReminderChoice, ReminderShowPayload, DailyStats } from "../shared/types";
import { playEntryPop, playYesDing } from "./sound";

export type SceneState =
  | "hidden"
  | "walk_in"
  | "idle"
  | "ask"
  | "happy"
  | "shrug"
  | "sad"
  | "wave"
  | "walk_out";

export type EntranceSide = "left" | "right";

export interface SoundPrefs {
  enabled: boolean;
  volume: number;
}

export interface DirectorCallbacks {
  onStateChange: (state: SceneState) => void;
  onLineChange: (line: string) => void;
  onButtonsVisibleChange: (visible: boolean) => void;
  onEntranceSideChange: (side: EntranceSide) => void;
  onGoalReached: () => void;
}

const PRAISE_LINES = ["Nice! 💙", "Yesss 💧", "That's the way!", "Hydration hero 💙"];

const TIMEOUT_MS = 25_000;
const SETTLE_MS = 300;
const WALK_MS = 1200;
const REDUCED_WALK_MS = 250; // still gives the crossfade in Stage.svelte room to play
const BUTTONS_DELAY_MS = 420; // after the last word pops in

/**
 * Runs one full reminder "scene": walk in, settle, ask, react to the user's
 * choice (or a timeout), then walk out. Owns no DOM — it only drives the
 * state the Stage/SvgCharacter/Bubble components render, and reports the
 * final choice back to Rust via `reminder_response`.
 */
export class Director {
  private reminderId = "";
  private snoozeMin = 10;
  private timeoutHandle: ReturnType<typeof setTimeout> | undefined;
  private resolved = false;
  private reducedMotion = false;
  private sound: SoundPrefs = { enabled: false, volume: 0.5 };

  constructor(private callbacks: DirectorCallbacks) {}

  setReducedMotion(enabled: boolean) {
    this.reducedMotion = enabled;
  }

  setSoundPrefs(prefs: SoundPrefs) {
    this.sound = prefs;
  }

  private wait(ms: number) {
    return new Promise<void>((resolve) => setTimeout(resolve, ms));
  }

  private walkWait() {
    return this.wait(this.reducedMotion ? REDUCED_WALK_MS : WALK_MS);
  }

  async playScene(payload: ReminderShowPayload, entrancePref: "left" | "right" | "random" = "random") {
    this.resolved = false;
    this.reminderId = payload.reminderId;
    this.snoozeMin = payload.snoozeMin;

    const side: EntranceSide =
      entrancePref === "random" ? (Math.random() < 0.5 ? "left" : "right") : entrancePref;
    this.callbacks.onEntranceSideChange(side);

    if (this.sound.enabled) playEntryPop(this.sound.volume);

    this.callbacks.onButtonsVisibleChange(false);
    this.callbacks.onStateChange("walk_in");
    await this.walkWait();

    this.callbacks.onStateChange("idle");
    await this.wait(SETTLE_MS);

    const line = payload.lines[Math.floor(Math.random() * payload.lines.length)] ?? "";
    this.callbacks.onLineChange(line);
    this.callbacks.onStateChange("ask");

    await this.wait(BUTTONS_DELAY_MS);
    this.callbacks.onButtonsVisibleChange(true);

    this.timeoutHandle = setTimeout(() => this.choose("timeout"), TIMEOUT_MS);
  }

  async choose(choice: ReminderChoice) {
    if (this.resolved) return;
    this.resolved = true;
    if (this.timeoutHandle) clearTimeout(this.timeoutHandle);

    this.callbacks.onButtonsVisibleChange(false);

    // Log the response immediately — before the reaction/walk-out
    // animation plays, not after — so stats are recorded even if
    // something interrupts the rest of the scene, and so a goal-reached
    // moment can be shown while the character is still on screen.
    let stats: DailyStats | null = null;
    try {
      stats = await invoke<DailyStats>("reminder_response", {
        payload: { reminderId: this.reminderId, choice },
      });
    } catch {
      // Scheduler/stats failure shouldn't block the scene from finishing.
    }

    const justReachedGoal =
      choice === "yes" && stats !== null && stats.goal > 0 && stats.glasses === stats.goal;

    if (choice === "yes") {
      if (this.sound.enabled) playYesDing(this.sound.volume);
      const praise = justReachedGoal
        ? `Goal done! ${stats!.glasses}/${stats!.goal} 🎉`
        : PRAISE_LINES[Math.floor(Math.random() * PRAISE_LINES.length)];
      this.callbacks.onLineChange(praise);
      this.callbacks.onStateChange("happy");
      if (justReachedGoal) this.callbacks.onGoalReached();
      await this.wait(900);
      this.callbacks.onStateChange("wave");
      await this.wait(500);
    } else if (choice === "later") {
      this.callbacks.onLineChange(`Okay, ${this.snoozeMin} minutes!`);
      this.callbacks.onStateChange("shrug");
      await this.wait(700);
    } else {
      // timeout or skip
      this.callbacks.onLineChange("I'll come back later…");
      this.callbacks.onStateChange("sad");
      await this.wait(900);
    }

    this.callbacks.onStateChange("walk_out");
    await this.walkWait();

    this.callbacks.onStateChange("hidden");
    this.callbacks.onLineChange("");

    await invoke("end_scene");
  }
}
