mod commands;
mod idle;
mod overlay;
mod packs;
mod reminders;
mod scheduler;
mod settings;
mod stats;
mod tray;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Another instance was launched; focus the settings window instead of
            // starting a second copy.
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::set_hit_regions,
            commands::overlay_ready,
            commands::reminder_response,
            commands::end_scene,
            commands::trigger_test_reminder,
            commands::pause_reminders,
            commands::resume_reminders,
            commands::set_dnd,
            commands::scheduler_status,
            settings::get_reminders,
            settings::save_reminders,
            settings::get_general_prefs,
            settings::save_general_prefs,
            settings::get_character_prefs,
            settings::save_character_prefs,
            settings::get_daily_goal,
            settings::save_daily_goal,
            settings::get_stats_snapshot,
            settings::open_data_folder,
            settings::reset_all_data,
            settings::app_version,
            packs::list_packs,
            packs::import_pack,
            packs::list_catalog,
            packs::download_pack,
        ])
        .manage(scheduler::SchedulerState::default())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Settings window: a normal decorated window, created hidden and only
            // shown on demand from the tray menu.
            WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("/".into()))
                .title("Companion Settings")
                .inner_size(720.0, 520.0)
                .resizable(true)
                .visible(false)
                .build()?;

            overlay::create(app.handle())?;

            tray::setup_tray(app.handle())?;

            scheduler::start(app.handle());

            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the settings window just hides it — the tray keeps the app alive.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "settings" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
