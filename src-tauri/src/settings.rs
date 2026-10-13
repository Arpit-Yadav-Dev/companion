//! Commands backing the Settings window: reading/writing reminders and
//! user prefs (both live in `settings.json` via tauri-plugin-store),
//! autostart, stats queries, and the About tab's data-folder/reset-all
//! actions.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

use crate::reminders::Reminder;
use crate::stats::{self, DailyStats};

const SETTINGS_STORE: &str = "settings.json";
const STATS_STORE: &str = "stats.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralPrefs {
    pub user_name: String,
    pub launch_at_login: bool,
    pub start_paused: bool,
    pub dnd: bool,
    pub reduced_motion: bool,
    pub sound_enabled: bool,
    pub sound_volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterPrefs {
    pub active_pack_id: String,
    pub theme: String,
    pub size_percent: u32,
    pub entrance_side: String, // "left" | "right" | "random"
    pub position: String,     // "bottom_left" | "bottom_right" | "bottom_center"
    #[serde(default = "default_animation_speed_percent")]
    pub animation_speed_percent: u32, // scales walk/leg/gesture animation speed; 100 = normal
}

fn default_animation_speed_percent() -> u32 {
    100
}

fn default_general() -> GeneralPrefs {
    GeneralPrefs {
        user_name: "there".into(),
        launch_at_login: false,
        start_paused: false,
        dnd: false,
        reduced_motion: false,
        sound_enabled: false,
        // Deliberately quiet. A reminder interrupts whatever you're doing,
        // so the sound should register without being startling — someone
        // can always turn it up, but a loud first impression is the kind
        // of thing people disable rather than adjust.
        sound_volume: 0.25,
    }
}

fn default_character() -> CharacterPrefs {
    CharacterPrefs {
        active_pack_id: "sip".into(),
        theme: "blue".into(),
        size_percent: 100,
        entrance_side: "random".into(),
        position: "bottom_left".into(),
        animation_speed_percent: 100,
    }
}

#[tauri::command]
pub fn get_reminders<R: Runtime>(app: AppHandle<R>) -> Vec<Reminder> {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        return crate::reminders::built_in_defaults();
    };
    let saved: Vec<Reminder> = store
        .get("reminders")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    if saved.is_empty() {
        return crate::reminders::built_in_defaults();
    }

    // The saved list replaces the defaults wholesale, so built-ins added in
    // a later version would never reach anyone who had already run the app.
    // Append the ones they've never seen — disabled, like any new built-in,
    // so this adds options rather than new interruptions. Reminders the
    // user deliberately deleted are custom ones, which are never in this
    // list, so nothing they removed comes back.
    let mut merged = saved;
    let known: std::collections::HashSet<&str> = merged.iter().map(|r| r.id.as_str()).collect();
    let missing: Vec<Reminder> = crate::reminders::built_in_defaults()
        .into_iter()
        .filter(|d| !known.contains(d.id.as_str()))
        .map(|mut d| {
            d.enabled = false;
            d
        })
        .collect();

    if !missing.is_empty() {
        merged.extend(missing);
        store.set("reminders", serde_json::to_value(&merged).unwrap());
        let _ = store.save();
    }
    merged
}

#[tauri::command]
pub fn save_reminders<R: Runtime>(app: AppHandle<R>, reminders: Vec<Reminder>) {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        eprintln!("companion: settings store unwritable; change not saved");
        return;
    };
    store.set("reminders", serde_json::to_value(&reminders).unwrap());
    let _ = store.save();
    crate::scheduler::reload_reminders(&app);
}

#[tauri::command]
pub fn get_general_prefs<R: Runtime>(app: AppHandle<R>) -> GeneralPrefs {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        return default_general();
    };
    store
        .get("general")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(default_general)
}

#[tauri::command]
pub fn save_general_prefs<R: Runtime>(app: AppHandle<R>, prefs: GeneralPrefs) {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        eprintln!("companion: settings store unwritable; change not saved");
        return;
    };
    store.set("general", serde_json::to_value(&prefs).unwrap());
    let _ = store.save();

    let autolaunch = app.autolaunch();
    let is_enabled = autolaunch.is_enabled().unwrap_or(false);
    if prefs.launch_at_login && !is_enabled {
        let _ = autolaunch.enable();
    } else if !prefs.launch_at_login && is_enabled {
        let _ = autolaunch.disable();
    }

    crate::scheduler::set_dnd(&app, prefs.dnd);
    crate::scheduler::set_user_name(&app, prefs.user_name);
}

#[tauri::command]
pub fn get_character_prefs<R: Runtime>(app: AppHandle<R>) -> CharacterPrefs {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        return default_character();
    };
    store
        .get("character")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(default_character)
}

#[tauri::command]
pub fn save_character_prefs<R: Runtime>(app: AppHandle<R>, prefs: CharacterPrefs) {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        eprintln!("companion: settings store unwritable; change not saved");
        return;
    };
    store.set("character", serde_json::to_value(&prefs).unwrap());
    let _ = store.save();
}

#[tauri::command]
pub fn get_daily_goal<R: Runtime>(app: AppHandle<R>) -> u32 {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        return 8;
    };
    store
        .get("dailyGoal")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(8)
}

#[tauri::command]
pub fn save_daily_goal<R: Runtime>(app: AppHandle<R>, goal: u32) {
    let Ok(store) = app.store(SETTINGS_STORE) else {
        eprintln!("companion: settings store unwritable; change not saved");
        return;
    };
    store.set("dailyGoal", goal);
    let _ = store.save();
    crate::scheduler::set_daily_goal(&app, goal);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    pub today: DailyStats,
    pub last_seven_days: Vec<DailyStats>,
    pub current_streak: u32,
    pub best_streak: u32,
}

#[tauri::command]
pub fn get_stats_snapshot<R: Runtime>(app: AppHandle<R>) -> StatsSnapshot {
    let goal = get_daily_goal(app.clone());
    let today = stats::today(&app, goal).unwrap_or(DailyStats {
        date: String::new(),
        glasses: 0,
        goal,
        yes_count: 0,
        later_count: 0,
        missed_count: 0,
    });
    let last_seven_days = stats::recent_days(&app, 7, goal).unwrap_or_default();
    let (current_streak, best_streak) = stats::streaks(&app, goal).unwrap_or((0, 0));

    StatsSnapshot {
        today,
        last_seven_days,
        current_streak,
        best_streak,
    }
}

#[tauri::command]
pub fn open_data_folder<R: Runtime>(app: AppHandle<R>) {
    if let Ok(dir) = app.path().app_data_dir() {
        let _ = app.opener().reveal_item_in_dir(dir);
    }
}

#[tauri::command]
pub fn reset_all_data<R: Runtime>(app: AppHandle<R>) {
    if let Ok(store) = app.store(SETTINGS_STORE) {
        store.clear();
        let _ = store.save();
    }
    if let Ok(store) = app.store(STATS_STORE) {
        store.clear();
        let _ = store.save();
    }
    crate::scheduler::reload_reminders(&app);
}

#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
