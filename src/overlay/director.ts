import { invoke } from "@tauri-apps/api/core";
import type { ReminderChoice, ReminderShowPayload } from "../shared/types";

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

export interface DirectorCallbacks {
  onStateChange: (state: SceneState) => void;
  onLineChange: (line: string) => void;
  onButtonsVisibleChange: (visible: boolean) => void;
  onEntranceSideChange: (side: EntranceSide) => void;
}

const PRAISE_LINES = ["Nice! 💙", "Yesss 💧", "That's the way!", "Hydration hero 💙"];

const TIMEOUT_MS = 25_000;
const SETTLE_MS = 300;
const WALK_MS = 1200;
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

  constructor(private callbacks: DirectorCallbacks) {}

  private wait(ms: number) {
    return new Promise<void>((resolve) => setTimeout(resolve, ms));
  }

  async playScene(payload: ReminderShowPayload, entrancePref: "left" | "right" | "random" = "random") {
    this.resolved = false;
    this.reminderId = payload.reminderId;
    this.snoozeMin = payload.snoozeMin;

    const side: EntranceSide =
      entrancePref === "random" ? (Math.random() < 0.5 ? "left" : "right") : entrancePref;
    this.callbacks.onEntranceSideChange(side);

    this.callbacks.onButtonsVisibleChange(false);
    this.callbacks.onStateChange("walk_in");
    await this.wait(WALK_MS);

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

    if (choice === "yes") {
      const praise = PRAISE_LINES[Math.floor(Math.random() * PRAISE_LINES.length)];
      this.callbacks.onLineChange(praise);
      this.callbacks.onStateChange("happy");
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
    await this.wait(WALK_MS);

    this.callbacks.onStateChange("hidden");
    this.callbacks.onLineChange("");

    await invoke("reminder_response", {
      payload: { reminderId: this.reminderId, choice },
    });
  }
}
