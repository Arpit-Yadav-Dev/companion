mod commands;
mod overlay;
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
        .invoke_handler(tauri::generate_handler![
            commands::set_hit_regions,
            commands::overlay_ready,
            commands::reminder_response,
        ])
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
