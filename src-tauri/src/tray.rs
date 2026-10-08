use std::time::Duration;

use chrono::{DateTime, Local};
use tauri::{
    menu::{MenuBuilder, MenuItem, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime,
};

use crate::scheduler::{self, PauseDuration};

/// Handles to the two live-updating tray menu items, kept in managed state
/// so the periodic updater (and any scheduling change) can refresh their
/// text without re-building the whole menu.
pub struct TrayHandles<R: Runtime> {
    pub status_item: MenuItem<R>,
    pub next_at_item: MenuItem<R>,
}

/// Builds the static tray menu described in the brief and attaches it to a
/// template tray icon, then starts a 15s ticker that refreshes the
/// "Active"/"Paused" status and the "Next at …" countdown — cheap enough
/// to run even while the menu isn't open, per the brief (live-updating
/// only while the menu is open isn't something Tauri can detect).
pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let status_item = MenuItemBuilder::with_id("status", "● Active")
        .enabled(false)
        .build(app)?;

    let next_at_item = MenuItemBuilder::with_id("next_at", "Next at — …")
        .enabled(false)
        .build(app)?;

    let settings_item = MenuItemBuilder::with_id("settings", "Settings…").build(app)?;

    let pause_submenu = SubmenuBuilder::new(app, "Pause reminders")
        .text("pause_30m", "30 min")
        .text("pause_1h", "1 hour")
        .text("pause_tomorrow", "Until tomorrow")
        .separator()
        .text("pause_resume", "Resume")
        .build()?;

    let test_reminder_item =
        MenuItemBuilder::with_id("test_reminder", "Test reminder now").build(app)?;

    let quit_item = MenuItemBuilder::with_id("quit", "Quit Companion").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&status_item)
        .item(&next_at_item)
        .separator()
        .item(&settings_item)
        .item(&pause_submenu)
        .item(&test_reminder_item)
        .separator()
        .item(&quit_item)
        .build()?;

    TrayIconBuilder::with_id("companion-tray")
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("bundled tray icon must decode")
        }))
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| handle_menu_event(app, event.id.as_ref()))
        .build(app)?;

    app.manage(TrayHandles {
        status_item: status_item.clone(),
        next_at_item: next_at_item.clone(),
    });

    refresh_tray_text(app);
    start_tray_ticker(app.clone());

    Ok(())
}

fn start_tray_ticker<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(15)).await;
            refresh_tray_text(&app);
        }
    });
}

/// Updates the status/next-at menu item text from the scheduler's current
/// state. Called on a timer and immediately after any action that changes
/// scheduling (pause/resume/DND toggle).
pub fn refresh_tray_text<R: Runtime>(app: &AppHandle<R>) {
    let Some(handles) = app.try_state::<TrayHandles<R>>() else {
        return;
    };
    let status = scheduler::status(app);

    let status_text = if status.active {
        "● Active"
    } else {
        "⏸ Paused"
    };
    let _ = handles.status_item.set_text(status_text);

    let next_at_text = match (&status.next_reminder_id, &status.next_fire_at) {
        (Some(_), Some(fire_at)) => match DateTime::parse_from_rfc3339(fire_at) {
            Ok(dt) => format_next_at(dt.with_timezone(&Local)),
            Err(_) => "Next at — …".to_string(),
        },
        _ => "Next at — (no reminders enabled)".to_string(),
    };
    let _ = handles.next_at_item.set_text(next_at_text);
}

fn format_next_at(fire_at: DateTime<Local>) -> String {
    let now = Local::now();
    let remaining = fire_at - now;
    let time_str = fire_at.format("%-I:%M %p").to_string();

    if remaining.num_seconds() <= 0 {
        format!("Next at {time_str} · due now")
    } else {
        let mins = remaining.num_minutes();
        let secs = remaining.num_seconds() % 60;
        format!("Next at {time_str} · {mins}m {secs}s")
    }
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "settings" => {
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "test_reminder" => {
            scheduler::trigger_now(app, None);
        }
        "pause_30m" => {
            scheduler::pause(app, PauseDuration::ThirtyMin);
            refresh_tray_text(app);
        }
        "pause_1h" => {
            scheduler::pause(app, PauseDuration::OneHour);
            refresh_tray_text(app);
        }
        "pause_tomorrow" => {
            scheduler::pause(app, PauseDuration::UntilTomorrow);
            refresh_tray_text(app);
        }
        "pause_resume" => {
            scheduler::resume(app);
            refresh_tray_text(app);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
