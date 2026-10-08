// Typed IPC payloads shared conceptually with the Rust side (src-tauri/src/commands.rs).
// Keep field names in sync with the serde structs there.

export type ReminderChoice = "yes" | "later" | "skip" | "timeout";

export interface ReminderShowPayload {
  reminderId: string;
  emoji: string;
  lines: string[];
  yesLabel: string;
  laterLabel: string;
  snoozeMin: number;
}

export interface ReminderResponsePayload {
  reminderId: string;
  choice: ReminderChoice;
}

/** A rect in the overlay webview's own CSS pixel coordinate space. */
export interface HitRect {
  x: number;
  y: number;
  width: number;
  height: number;
}
