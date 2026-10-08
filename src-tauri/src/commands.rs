use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::overlay;
use crate::scheduler::{self, PauseDuration, SchedulerStatus};
use crate::stats::DailyStats;
use crate::tray;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HitRectPayload {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderShowPayload {
    pub reminder_id: String,
    pub emoji: String,
    pub lines: Vec<String>,
    pub yes_label: String,
    pub later_label: String,
    pub snooze_min: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderResponsePayload {
    pub reminder_id: String,
    pub choice: String,
}

/// Called by the overlay frontend whenever its interactive rects (character
/// body, bubble, buttons) change, so the Rust-side cursor-polling loop knows
/// where clicks should be allowed through.
#[tauri::command]
pub fn set_hit_regions<R: Runtime>(app: AppHandle<R>, rects: Vec<HitRectPayload>) {
    let rects = rects
        .into_iter()
        .map(|r| overlay::HitRect {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        })
        .collect();
    overlay::set_hit_regions(&app, rects);
}

/// Called by the overlay frontend once its first frame has painted, so we
/// only reveal the (until-now transparent/blank) window after there's
/// something to show — avoids a white/blank flash.
#[tauri::command]
pub fn overlay_ready<R: Runtime>(app: AppHandle<R>) {
    let _ = overlay::show(&app);
}

/// Called by the overlay frontend as soon as the user chooses
/// (yes/later/timeout/skip) — before the happy/sad/shrug reaction and
/// walk-out play, not after. Logs the response to stats, reschedules the
/// reminder, and returns the day's updated stats so the frontend can
/// show a special "goal reached" moment (confetti) while the character
/// is still on screen, before `end_scene` hides the window.
#[tauri::command]
pub fn reminder_response<R: Runtime>(app: AppHandle<R>, payload: ReminderResponsePayload) -> DailyStats {
    let stats = scheduler::handle_response(&app, &payload.reminder_id, &payload.choice);
    tray::refresh_tray_text(&app);
    stats
}

/// Called once the director's walk-out animation has finished and the
/// scene is fully done — hides the overlay window and stops the
/// hit-region polling loop.
#[tauri::command]
pub fn end_scene<R: Runtime>(app: AppHandle<R>) {
    let _ = overlay::hide(&app);
}

/// Wired to the tray's "Test reminder now" item and the Settings "Test"
/// button per reminder: fires a reminder immediately, bypassing active
/// hours/pause/DND.
#[tauri::command]
pub fn trigger_test_reminder<R: Runtime>(app: AppHandle<R>, reminder_id: Option<String>) {
    scheduler::trigger_now(&app, reminder_id.as_deref());
}

#[tauri::command]
pub fn pause_reminders<R: Runtime>(app: AppHandle<R>, duration: String) {
    let duration = match duration.as_str() {
        "thirty_min" => PauseDuration::ThirtyMin,
        "one_hour" => PauseDuration::OneHour,
        _ => PauseDuration::UntilTomorrow,
    };
    scheduler::pause(&app, duration);
    tray::refresh_tray_text(&app);
}

#[tauri::command]
pub fn resume_reminders<R: Runtime>(app: AppHandle<R>) {
    scheduler::resume(&app);
    tray::refresh_tray_text(&app);
}

#[tauri::command]
pub fn set_dnd<R: Runtime>(app: AppHandle<R>, enabled: bool) {
    scheduler::set_dnd(&app, enabled);
    tray::refresh_tray_text(&app);
}

#[tauri::command]
pub fn scheduler_status<R: Runtime>(app: AppHandle<R>) -> SchedulerStatus {
    scheduler::status(&app)
}
