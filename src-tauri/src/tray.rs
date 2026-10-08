use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime,
};

/// Builds the static tray menu described in the brief and attaches it to a
/// template tray icon. Dynamic bits (the "Next at …" countdown, the live
/// "Active"/"Paused" status) are wired up in Phase 4 once the scheduler
/// exists; for now the items are placeholders with stable ids so the
/// scheduler can update their text later via `MenuItemKind::as_normal_mut`.
pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let status_item = MenuItemBuilder::with_id("status", "● Active")
        .enabled(false)
        .build(app)?;

    let next_at_item = MenuItemBuilder::with_id("next_at", "Next at — (no reminders enabled)")
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

    let test_reminder_item = MenuItemBuilder::with_id("test_reminder", "Test reminder now").build(app)?;

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

    Ok(())
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
            // Phase 2 will wire this to the overlay's "show a placeholder
            // reminder" flow. No-op for now.
        }
        "pause_30m" | "pause_1h" | "pause_tomorrow" | "pause_resume" => {
            // Phase 4 (scheduler) will implement pausing.
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
