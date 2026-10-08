use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, Runtime, WebviewUrl,
    WebviewWindowBuilder,
};

pub const OVERLAY_LABEL: &str = "overlay";

/// A rect in the overlay webview's own CSS-pixel coordinate space, reported
/// by the frontend whenever its interactive elements (character body,
/// bubble, buttons) move or resize. Used to decide when to stop ignoring
/// cursor events so clicks reach those elements instead of passing through
/// to whatever app is behind the overlay.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HitRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Shared state for the hit-region polling loop: the latest rects reported
/// by the frontend (in logical/CSS pixels) plus whether we're currently
/// click-through (ignoring cursor events) so we only call the (somewhat
/// expensive) platform API when the state actually needs to flip.
pub struct OverlayState {
    rects: Mutex<Vec<HitRect>>,
    ignoring: AtomicBool,
    polling: AtomicBool,
    /// Logical-pixel origin of the overlay window on its current monitor,
    /// needed to convert the global cursor position into webview-local
    /// coordinates that can be compared against `rects`.
    origin_x: AtomicI32,
    origin_y: AtomicI32,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            rects: Mutex::new(Vec::new()),
            ignoring: AtomicBool::new(true),
            polling: AtomicBool::new(false),
            origin_x: AtomicI32::new(0),
            origin_y: AtomicI32::new(0),
        }
    }
}

/// Creates the overlay window, hidden, sized to the work area of whatever
/// monitor the cursor is on right now. It gets repositioned to the
/// cursor's monitor again each time it's shown (see [`show_on_cursor_monitor`]).
pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    app.manage(OverlayState::default());

    let window = WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("overlay".into()))
        .title("Companion Overlay")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()?;

    // Default to fully click-through until something is shown and reports
    // interactive rects.
    let _ = window.set_ignore_cursor_events(true);

    Ok(())
}

/// Positions and sizes the overlay to cover the work area (screen minus
/// Dock/menu bar) of the monitor the cursor is currently on: full width, the
/// bottom ~55% of the work area height. Falls back to the primary monitor's
/// full size minus a safe margin if work-area/monitor lookup fails.
pub fn position_on_cursor_monitor<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(OVERLAY_LABEL)
        .expect("overlay window must exist");

    let cursor = window.cursor_position()?;
    let monitor = window
        .monitor_from_point(cursor.x, cursor.y)?
        .or(window.primary_monitor()?);

    let state = app.state::<OverlayState>();

    if let Some(monitor) = monitor {
        let work_area = monitor.work_area();
        let scale = monitor.scale_factor();

        let strip_height_physical = (work_area.size.height as f64 * 0.55) as u32;
        let strip_y_physical =
            work_area.position.y + (work_area.size.height as i32 - strip_height_physical as i32);

        window.set_position(PhysicalPosition::new(
            work_area.position.x,
            strip_y_physical,
        ))?;
        window.set_size(tauri::PhysicalSize::new(
            work_area.size.width,
            strip_height_physical,
        ))?;

        let origin_logical = PhysicalPosition::new(work_area.position.x, strip_y_physical)
            .to_logical::<i32>(scale);
        state.origin_x.store(origin_logical.x, Ordering::Relaxed);
        state.origin_y.store(origin_logical.y, Ordering::Relaxed);
    } else {
        // No monitor info available — fall back to a fixed-size strip
        // anchored to a safe margin from the window's current screen.
        let margin = 40i32;
        window.set_size(LogicalSize::new(1200.0, 500.0))?;
        window.set_position(LogicalPosition::new(margin, margin))?;
        state.origin_x.store(margin, Ordering::Relaxed);
        state.origin_y.store(margin, Ordering::Relaxed);
    }

    Ok(())
}

pub fn show<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    position_on_cursor_monitor(app)?;
    let window = app
        .get_webview_window(OVERLAY_LABEL)
        .expect("overlay window must exist");
    window.show()?;
    start_hit_region_polling(app.clone());
    Ok(())
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(OVERLAY_LABEL)
        .expect("overlay window must exist");
    window.hide()?;
    let _ = window.set_ignore_cursor_events(true);
    let state = app.state::<OverlayState>();
    state.ignoring.store(true, Ordering::Relaxed);
    state.polling.store(false, Ordering::Relaxed);
    Ok(())
}

/// Called from the frontend (`set_hit_regions` command) whenever the
/// interactive rects change — the character body, the speech bubble, and
/// its buttons.
pub fn set_hit_regions<R: Runtime>(app: &AppHandle<R>, rects: Vec<HitRect>) {
    let state = app.state::<OverlayState>();
    *state.rects.lock().unwrap() = rects;
}

/// Spawns (if not already running) a background thread that polls the
/// global cursor position roughly every 33ms while the overlay is visible.
/// When the cursor enters one of the reported hit rects we flip the window
/// to accept clicks; when it leaves all of them we go back to click-through.
/// The loop exits as soon as `hide` flips `polling` back to false, so there
/// is zero ongoing cost while the overlay is hidden.
fn start_hit_region_polling<R: Runtime>(app: AppHandle<R>) {
    let state = app.state::<OverlayState>();
    if state.polling.swap(true, Ordering::Relaxed) {
        return; // already running
    }

    thread::spawn(move || {
        loop {
            let state = app.state::<OverlayState>();
            if !state.polling.load(Ordering::Relaxed) {
                break;
            }

            let Some(window) = app.get_webview_window(OVERLAY_LABEL) else {
                break;
            };

            if let Ok(cursor) = window.cursor_position() {
                let origin_x = state.origin_x.load(Ordering::Relaxed) as f64;
                let origin_y = state.origin_y.load(Ordering::Relaxed) as f64;
                let scale = window.scale_factor().unwrap_or(1.0);

                // cursor_position() is in physical pixels; convert to the
                // overlay's logical/CSS coordinate space before comparing
                // against the rects the frontend reported.
                let local_x = cursor.x / scale - origin_x;
                let local_y = cursor.y / scale - origin_y;

                let inside = state
                    .rects
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|r| {
                        local_x >= r.x
                            && local_x <= r.x + r.width
                            && local_y >= r.y
                            && local_y <= r.y + r.height
                    });

                let was_ignoring = state.ignoring.load(Ordering::Relaxed);
                if inside && was_ignoring {
                    let _ = window.set_ignore_cursor_events(false);
                    state.ignoring.store(false, Ordering::Relaxed);
                } else if !inside && !was_ignoring {
                    let _ = window.set_ignore_cursor_events(true);
                    state.ignoring.store(true, Ordering::Relaxed);
                }
            }

            thread::sleep(Duration::from_millis(33));
        }
    });
}
