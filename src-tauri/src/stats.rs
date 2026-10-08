//! Daily reminder logs and streaks, persisted via tauri-plugin-store.

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "stats.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStats {
    pub date: String, // YYYY-MM-DD
    pub glasses: u32,
    pub goal: u32,
    pub yes_count: u32,
    pub later_count: u32,
    pub missed_count: u32,
}

impl DailyStats {
    fn new(date: NaiveDate, goal: u32) -> Self {
        Self {
            date: date.format("%Y-%m-%d").to_string(),
            glasses: 0,
            goal,
            yes_count: 0,
            later_count: 0,
            missed_count: 0,
        }
    }
}

fn today_key() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn load_day<R: Runtime>(app: &AppHandle<R>, date: &str, default_goal: u32) -> tauri::Result<DailyStats> {
    let store = app.store(STORE_FILE).map_err(|e| {
        tauri::Error::Anyhow(anyhow::anyhow!("failed to open stats store: {e}"))
    })?;

    let key = format!("day:{date}");
    if let Some(value) = store.get(&key) {
        if let Ok(stats) = serde_json::from_value::<DailyStats>(value) {
            return Ok(stats);
        }
    }

    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("bad date key: {e}")))?;
    Ok(DailyStats::new(parsed_date, default_goal))
}

fn save_day<R: Runtime>(app: &AppHandle<R>, stats: &DailyStats) -> tauri::Result<()> {
    let store = app.store(STORE_FILE).map_err(|e| {
        tauri::Error::Anyhow(anyhow::anyhow!("failed to open stats store: {e}"))
    })?;
    let key = format!("day:{}", stats.date);
    store.set(key, serde_json::to_value(stats).unwrap());
    store.save().map_err(|e| {
        tauri::Error::Anyhow(anyhow::anyhow!("failed to save stats store: {e}"))
    })
}

/// Records one reminder response against today's stats. `is_water` controls
/// whether a "yes" also increments the glasses count (only the water
/// reminder logs glasses; other reminder types just track yes/later/missed).
pub fn record_response<R: Runtime>(
    app: &AppHandle<R>,
    choice: &str,
    is_water: bool,
    default_goal: u32,
) -> tauri::Result<DailyStats> {
    let date = today_key();
    let mut stats = load_day(app, &date, default_goal)?;

    match choice {
        "yes" => {
            stats.yes_count += 1;
            if is_water {
                stats.glasses += 1;
            }
        }
        "later" => stats.later_count += 1,
        _ => stats.missed_count += 1, // timeout or skip
    }

    save_day(app, &stats)?;
    Ok(stats)
}

pub fn today<R: Runtime>(app: &AppHandle<R>, default_goal: u32) -> tauri::Result<DailyStats> {
    load_day(app, &today_key(), default_goal)
}

/// Returns stats for the last `days` days (oldest first), for the 7-day bar
/// chart. Missing days are filled with empty stats rather than erroring.
/// Not called yet — wired up to the Settings Stats tab in Phase 5.
#[allow(dead_code)]
pub fn recent_days<R: Runtime>(
    app: &AppHandle<R>,
    days: i64,
    default_goal: u32,
) -> tauri::Result<Vec<DailyStats>> {
    let today = Local::now().date_naive();
    let mut out = Vec::with_capacity(days as usize);
    for i in (0..days).rev() {
        let date = today - chrono::Duration::days(i);
        out.push(load_day(app, &date.format("%Y-%m-%d").to_string(), default_goal)?);
    }
    Ok(out)
}

/// Current and best streak of days where the water goal was met, scanning
/// up to a year back. The current streak is the run of met days ending
/// today (0 if today hasn't met goal yet, since the day isn't over); best
/// is the longest such run anywhere in the scanned history.
pub fn streaks<R: Runtime>(app: &AppHandle<R>, default_goal: u32) -> tauri::Result<(u32, u32)> {
    let today = Local::now().date_naive();
    let mut current = 0u32;
    let mut best = 0u32;
    let mut running = 0u32;
    let mut still_counting_current = true;

    for i in 0..365 {
        let date = today - chrono::Duration::days(i);
        let stats = load_day(app, &date.format("%Y-%m-%d").to_string(), default_goal)?;
        let met = stats.goal > 0 && stats.glasses >= stats.goal;

        if met {
            running += 1;
            best = best.max(running);
        } else {
            if still_counting_current {
                current = running;
                still_counting_current = false;
            }
            running = 0;
        }
    }
    if still_counting_current {
        current = running;
    }

    Ok((current, best))
}
