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
  /** Which reminder-specific pose a character should use for its "ask"
   * state, e.g. "bottle" | "stretch" | "lookfar" | "posture". Undefined
   * falls back to the character's generic ask pose. */
  prop?: string | null;
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

export interface ActiveHours {
  startMinute: number; // minutes since midnight, local time
  endMinute: number;
}

/** 0 = Sunday ... 6 = Saturday, matching chrono's Weekday::num_days_from_sunday(). */
export type DayOfWeek = 0 | 1 | 2 | 3 | 4 | 5 | 6;

export interface Reminder {
  id: string;
  enabled: boolean;
  emoji: string;
  title: string;
  intervalMin: number;
  activeHours: ActiveHours;
  daysOfWeek: DayOfWeek[];
  lines: string[];
  yesLabel: string;
  laterLabel: string;
  snoozeMin: number;
  prop: string | null;
}

export type PauseDuration = "thirty_min" | "one_hour" | "until_tomorrow";

export interface SchedulerStatus {
  active: boolean;
  pausedUntil: string | null; // ISO 8601, or null if not paused
  nextReminderId: string | null;
  nextFireAt: string | null; // ISO 8601
}

export interface DailyStats {
  date: string; // YYYY-MM-DD, local
  glasses: number;
  goal: number;
  yesCount: number;
  laterCount: number;
  missedCount: number;
}

export interface GeneralPrefs {
  userName: string;
  launchAtLogin: boolean;
  startPaused: boolean;
  dnd: boolean;
  reducedMotion: boolean;
  soundEnabled: boolean;
  soundVolume: number;
}

export type EntranceSidePref = "left" | "right" | "random";
export type PositionPref = "bottom_left" | "bottom_right" | "bottom_center";
export type ThemeName = "blue" | "peach" | "mint" | "night";

export interface CharacterPrefs {
  activePackId: string;
  theme: ThemeName;
  sizePercent: number;
  entranceSide: EntranceSidePref;
  position: PositionPref;
  /** Scales walk/leg/gesture animation speed; 100 = normal. */
  animationSpeedPercent: number;
}

export interface StatsSnapshot {
  today: DailyStats;
  lastSevenDays: DailyStats[];
  currentStreak: number;
  bestStreak: number;
}

export interface PackStateConfig {
  dir: string;
  loop: boolean;
  move?: "enter" | "exit";
}

export interface PackManifest {
  id: string;
  name: string;
  renderer: "svg" | "frames";
  fps: number;
  heightPx: number;
  anchor: { x: number; y: number };
  bubbleOffset: { x: number; y: number };
  states: Record<string, PackStateConfig>;
  props: Record<string, string>;
  mirrorForDirection: boolean;
  stridePx?: number;
}

export interface PackInfo {
  manifest: PackManifest;
  dir: string;
}
