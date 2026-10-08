use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};

use crate::overlay;

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

/// Called by the overlay frontend when the director finishes a scene
/// (yes/later/timeout/skip). For Phase 2 this just hides the overlay again;
/// the scheduler (Phase 4) will log the response and reschedule.
#[tauri::command]
pub fn reminder_response<R: Runtime>(app: AppHandle<R>, payload: ReminderResponsePayload) {
    eprintln!(
        "reminder {} resolved with choice {}",
        payload.reminder_id, payload.choice
    );
    let _ = overlay::hide(&app);
}

/// Wired to the tray's "Test reminder now" item: shows the overlay with a
/// placeholder water reminder so the whole show/click-through/button flow
/// can be exercised without the scheduler.
pub fn trigger_test_reminder<R: Runtime>(app: &AppHandle<R>) {
    let payload = ReminderShowPayload {
        reminder_id: "test".into(),
        emoji: "💧".into(),
        lines: vec!["Hey — this is a test reminder!".into()],
        yes_label: "YES".into(),
        later_label: "Remind me later".into(),
        snooze_min: 10,
    };
    let _ = app.emit_to(overlay::OVERLAY_LABEL, "reminder://show", payload);
}
