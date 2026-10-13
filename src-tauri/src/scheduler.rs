//! Owns *when* reminders fire. Ticks once a second, decides whether any
//! enabled reminder is due (respecting active hours/days, pause, DND, and
//! idle-awareness), and if so emits `reminder://show` to the overlay and
//! waits for `reminder_response` before considering the next one — so two
//! reminders never show at once, and there's always at least a 90s gap.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use chrono::{Datelike, Local, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::idle;
use crate::overlay;
use crate::reminders::{built_in_defaults, Reminder};
use crate::stats;

const STORE_FILE: &str = "settings.json";
const MIN_GAP: Duration = Duration::from_secs(90);
const IDLE_PAUSE_THRESHOLD: Duration = Duration::from_secs(5 * 60);
const AWAY_RESET_THRESHOLD: Duration = Duration::from_secs(15 * 60);
const TICK_INTERVAL: Duration = Duration::from_secs(1);
/// A gap between 1Hz ticks larger than this means the process was
/// suspended (system sleep) rather than just scheduled late. Generous
/// enough that heavy load or a stalled runtime won't trip it.
const SLEEP_GAP_THRESHOLD_SECS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseDuration {
    ThirtyMin,
    OneHour,
    UntilTomorrow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerStatus {
    pub active: bool,
    pub paused_until: Option<String>,
    pub next_reminder_id: Option<String>,
    pub next_fire_at: Option<String>,
}

struct ReminderRuntime {
    /// Wall-clock time this reminder is next due. Reset on load, on
    /// response, and when settings change.
    next_due: chrono::DateTime<Local>,
    last_line_index: Option<usize>,
}

pub struct SchedulerState {
    inner: Mutex<Inner>,
}

struct Inner {
    reminders: Vec<Reminder>,
    runtime: HashMap<String, ReminderRuntime>,
    user_name: String,
    daily_goal: u32,
    dnd: bool,
    paused_until: Option<chrono::DateTime<Local>>,
    /// Set while a reminder is showing or within the post-response gap, so
    /// the tick loop won't start another one.
    busy_until: chrono::DateTime<Local>,
    awaiting_response_for: Option<String>,
    was_idle: bool,
    /// Wall-clock time of the previous tick, used to notice that the
    /// machine slept: ticks are 1s apart, so a much larger jump means the
    /// process was suspended rather than merely busy.
    last_tick: chrono::DateTime<Local>,
    /// The local date (YYYY-MM-DD) of the last reminder shown, so the
    /// first reminder of a new day can get the "Good morning" treatment.
    last_shown_date: Option<chrono::NaiveDate>,
}

impl Default for SchedulerState {
    fn default() -> Self {
        let now = Local::now();
        Self {
            inner: Mutex::new(Inner {
                reminders: Vec::new(),
                runtime: HashMap::new(),
                user_name: "there".into(),
                daily_goal: 8,
                dnd: false,
                paused_until: None,
                busy_until: now,
                awaiting_response_for: None,
                was_idle: false,
                last_tick: now,
                last_shown_date: None,
            }),
        }
    }
}

struct LoadedSettings {
    reminders: Vec<Reminder>,
    user_name: String,
    daily_goal: u32,
    dnd: bool,
    start_paused: bool,
}

fn load_settings<R: Runtime>(app: &AppHandle<R>) -> LoadedSettings {
    let store = app.store(STORE_FILE).expect("settings store must open");

    let reminders = store
        .get("reminders")
        .and_then(|v| serde_json::from_value::<Vec<Reminder>>(v).ok())
        .unwrap_or_else(|| {
            let defaults = built_in_defaults();
            store.set("reminders", serde_json::to_value(&defaults).unwrap());
            let _ = store.save();
            defaults
        });

    // GeneralPrefs (name, dnd, start-paused, ...) lives under the
    // "general" key as one nested object — see settings::GeneralPrefs /
    // settings::save_general_prefs, which is the single writer for it.
    let general = store.get("general");
    let user_name = general
        .as_ref()
        .and_then(|v| v.get("userName"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| "there".into());
    let dnd = general
        .as_ref()
        .and_then(|v| v.get("dnd"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let start_paused = general
        .as_ref()
        .and_then(|v| v.get("startPaused"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let daily_goal = store
        .get("dailyGoal")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(8);

    LoadedSettings {
        reminders,
        user_name,
        daily_goal,
        dnd,
        start_paused,
    }
}

/// Starts the scheduler: loads reminders from the store, seeds each
/// enabled reminder's next-due time to now + its interval, then spawns the
/// 1Hz tick loop. Call once from `setup`.
pub fn start<R: Runtime>(app: &AppHandle<R>) {
    let loaded = load_settings(app);

    let state = app.state::<SchedulerState>();
    {
        let mut inner = state.inner.lock().unwrap();
        inner.user_name = loaded.user_name;
        inner.daily_goal = loaded.daily_goal;
        inner.dnd = loaded.dnd;
        if loaded.start_paused {
            // "Start paused" per the General tab: pause indefinitely
            // rather than a fixed duration, since this isn't a daily
            // reset — far-future paused_until, cleared the same way any
            // other pause is (tray's "Resume" item).
            inner.paused_until = Some(Local::now() + chrono::Duration::days(3650));
        }
        let now = Local::now();
        for r in &loaded.reminders {
            inner.runtime.insert(
                r.id.clone(),
                ReminderRuntime {
                    next_due: now + chrono::Duration::minutes(r.interval_min as i64),
                    last_line_index: None,
                },
            );
        }
        inner.reminders = loaded.reminders;
    }

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(TICK_INTERVAL).await;
            tick(&app);
        }
    });
}

fn tick<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<SchedulerState>();
    let now = Local::now();

    // Idle-awareness: pause timers while idle > 5 min; if the user was
    // away > 15 min, reset (not immediately re-fire) the water timer so
    // they aren't nagged the moment they sit back down.
    let idle_for = idle::idle_duration();
    let display_asleep = idle::display_is_asleep();
    {
        let mut inner = state.inner.lock().unwrap();

        // Ticks are 1s apart, so a much larger gap means the process was
        // suspended — the machine slept. Wall-clock next_due times are all
        // long overdue on wake, and firing them immediately would greet the
        // user with a reminder (and log the rest as missed) for time they
        // were not at the desk. Push everything out by a fresh interval.
        //
        // The idle-based reset below can't cover this on its own: it only
        // triggers when the previous tick saw the user as idle, and sleeping
        // mid-use (shutting the lid while typing) leaves was_idle false.
        let slept_for = now.signed_duration_since(inner.last_tick);
        inner.last_tick = now;
        if slept_for > chrono::Duration::seconds(SLEEP_GAP_THRESHOLD_SECS) {
            for r in &inner.reminders.clone() {
                if let Some(rt) = inner.runtime.get_mut(&r.id) {
                    rt.next_due = now + chrono::Duration::minutes(r.interval_min as i64);
                }
            }
            inner.was_idle = false;
            return;
        }

        // A reminder shown to a dark screen is simply spent: the user never
        // sees it, yet it would still reset the timer and be logged as
        // missed. Hold the timers instead, exactly as for an idle user.
        if display_asleep {
            inner.was_idle = true;
            return;
        }

        let is_idle_now = idle_for.map(|d| d >= IDLE_PAUSE_THRESHOLD).unwrap_or(false);
        let was_away_long = idle_for.map(|d| d >= AWAY_RESET_THRESHOLD).unwrap_or(false);

        if inner.was_idle && !is_idle_now && was_away_long {
            // Just returned from a long absence: push every reminder's
            // next-due time out by its full interval instead of letting an
            // overdue one fire immediately.
            for r in &inner.reminders.clone() {
                if let Some(rt) = inner.runtime.get_mut(&r.id) {
                    rt.next_due = now + chrono::Duration::minutes(r.interval_min as i64);
                }
            }
        }
        inner.was_idle = is_idle_now;

        if is_idle_now {
            return; // timers effectively paused
        }

        if inner.dnd {
            return;
        }
        if let Some(until) = inner.paused_until {
            if now < until {
                return;
            }
            inner.paused_until = None;
        }
        if now < inner.busy_until {
            return;
        }
        if inner.awaiting_response_for.is_some() {
            return; // a reminder is already showing
        }
    }

    // Find the most overdue enabled reminder that's within its active
    // hours/days, if any.
    let due_reminder = {
        let inner = state.inner.lock().unwrap();
        inner
            .reminders
            .iter()
            .filter(|r| r.enabled)
            .filter(|r| is_within_active_window(r, now))
            .filter_map(|r| {
                inner
                    .runtime
                    .get(&r.id)
                    .filter(|rt| rt.next_due <= now)
                    .map(|rt| (r.clone(), rt.next_due))
            })
            .min_by_key(|(_, due)| *due)
            .map(|(r, _)| r)
    };

    let Some(reminder) = due_reminder else {
        return;
    };

    fire(app, &reminder);
}

fn is_within_active_window(reminder: &Reminder, now: chrono::DateTime<Local>) -> bool {
    let weekday = now.weekday().num_days_from_sunday() as u8;
    if !reminder.days_of_week.contains(&weekday) {
        return false;
    }
    let minute_of_day = now.hour() * 60 + now.minute();
    minute_of_day >= reminder.active_hours.start_minute
        && minute_of_day <= reminder.active_hours.end_minute
}

fn fire<R: Runtime>(app: &AppHandle<R>, reminder: &Reminder) {
    let state = app.state::<SchedulerState>();
    let mut inner = state.inner.lock().unwrap();

    let now = Local::now();
    let today = now.date_naive();
    let is_first_today = inner.last_shown_date != Some(today);
    inner.last_shown_date = Some(today);

    let line_index = pick_line_index(reminder, &mut inner, now);
    let template = &reminder.lines[line_index];

    let glasses = if template.contains("{count}") {
        stats::today(app, inner.daily_goal).map(|s| s.glasses).unwrap_or(0)
    } else {
        0
    };
    let streak = if template.contains("{streak}") {
        stats::streaks(app, inner.daily_goal).map(|(c, _)| c).unwrap_or(0)
    } else {
        0
    };
    let line = render_line(
        template,
        &inner.user_name,
        glasses,
        streak,
        is_first_today && now.hour() < 11,
    );

    inner.awaiting_response_for = Some(reminder.id.clone());

    let payload = crate::commands::ReminderShowPayload {
        reminder_id: reminder.id.clone(),
        emoji: reminder.emoji.clone(),
        lines: vec![line],
        yes_label: reminder.yes_label.clone(),
        later_label: reminder.later_label.clone(),
        snooze_min: reminder.snooze_min,
        prop: reminder.prop.clone(),
    };
    drop(inner);

    let _ = app.emit_to(overlay::OVERLAY_LABEL, "reminder://show", payload);
    let _ = overlay::show(app);
}

fn pick_line_index(reminder: &Reminder, inner: &mut Inner, _now: chrono::DateTime<Local>) -> usize {
    if reminder.lines.len() <= 1 {
        return 0;
    }
    let rt = inner.runtime.get_mut(&reminder.id);
    let last = rt.as_ref().and_then(|r| r.last_line_index);
    let mut idx = fastrand_index(reminder.lines.len());
    if Some(idx) == last {
        idx = (idx + 1) % reminder.lines.len();
    }
    if let Some(rt) = rt {
        rt.last_line_index = Some(idx);
    }
    idx
}

/// Tiny dependency-free random index — we don't need a crypto RNG for
/// "don't repeat the same line twice."
fn fastrand_index(len: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos as usize) % len
}

fn render_line(template: &str, name: &str, count: u32, streak: u32, with_morning_greeting: bool) -> String {
    let rendered = template
        .replace("{name}", name)
        .replace("{count}", &count.to_string())
        .replace("{streak}", &streak.to_string());

    if with_morning_greeting {
        format!("Good morning, {name}! {rendered}")
    } else {
        rendered
    }
}

/// Called from `commands::reminder_response` as soon as the overlay
/// reports a choice (before the reaction/walk-out animation plays).
/// Logs stats, reschedules the reminder (snoozed if "later", normal
/// interval otherwise), clears the busy/awaiting-response gate after the
/// minimum gap, and returns the day's updated stats.
pub fn handle_response<R: Runtime>(
    app: &AppHandle<R>,
    reminder_id: &str,
    choice: &str,
) -> stats::DailyStats {
    let state = app.state::<SchedulerState>();
    let now = Local::now();

    let (is_water, snooze_min, interval_min, daily_goal) = {
        let inner = state.inner.lock().unwrap();
        let reminder = inner.reminders.iter().find(|r| r.id == reminder_id).cloned();
        match reminder {
            Some(r) => (r.id == "water", r.snooze_min, r.interval_min, inner.daily_goal),
            None => (false, 10, 45, inner.daily_goal),
        }
    };

    let updated_stats = stats::record_response(app, choice, is_water, daily_goal)
        .unwrap_or_else(|_| stats::today(app, daily_goal).unwrap_or(stats::DailyStats {
            date: String::new(),
            glasses: 0,
            goal: daily_goal,
            yes_count: 0,
            later_count: 0,
            missed_count: 0,
        }));

    let mut inner = state.inner.lock().unwrap();
    inner.awaiting_response_for = None;
    inner.busy_until = now + MIN_GAP;

    let delay_min = if choice == "later" { snooze_min } else { interval_min };
    if let Some(rt) = inner.runtime.get_mut(reminder_id) {
        rt.next_due = now + chrono::Duration::minutes(delay_min as i64);
    }

    updated_stats
}

pub fn pause<R: Runtime>(app: &AppHandle<R>, duration: PauseDuration) {
    let state = app.state::<SchedulerState>();
    let mut inner = state.inner.lock().unwrap();
    let now = Local::now();
    inner.paused_until = Some(match duration {
        PauseDuration::ThirtyMin => now + chrono::Duration::minutes(30),
        PauseDuration::OneHour => now + chrono::Duration::hours(1),
        PauseDuration::UntilTomorrow => {
            let tomorrow = (now + chrono::Duration::days(1)).date_naive();
            tomorrow
                .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                .and_local_timezone(Local)
                .single()
                .unwrap_or(now + chrono::Duration::hours(24))
        }
    });
}

pub fn resume<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<SchedulerState>();
    let mut inner = state.inner.lock().unwrap();
    inner.paused_until = None;
}

pub fn set_dnd<R: Runtime>(app: &AppHandle<R>, enabled: bool) {
    let state = app.state::<SchedulerState>();
    let mut inner = state.inner.lock().unwrap();
    inner.dnd = enabled;
}

pub fn set_user_name<R: Runtime>(app: &AppHandle<R>, name: String) {
    let state = app.state::<SchedulerState>();
    let mut inner = state.inner.lock().unwrap();
    inner.user_name = name;
}

pub fn set_daily_goal<R: Runtime>(app: &AppHandle<R>, goal: u32) {
    let state = app.state::<SchedulerState>();
    let mut inner = state.inner.lock().unwrap();
    inner.daily_goal = goal;
}

/// Re-reads the reminders list from the store (after the Settings window
/// saves changes, or after a reset-all) and re-seeds next-due times for
/// any reminder that doesn't already have one, so edits take effect live
/// without needing a restart. Existing next-due times for reminders that
/// already existed are left alone, so editing a reminder's lines or
/// labels doesn't reset its countdown.
pub fn reload_reminders<R: Runtime>(app: &AppHandle<R>) {
    let loaded = load_settings(app);
    let reminders = loaded.reminders;
    let state = app.state::<SchedulerState>();
    let mut inner = state.inner.lock().unwrap();
    let now = Local::now();

    inner
        .runtime
        .retain(|id, _| reminders.iter().any(|r| &r.id == id));
    for r in &reminders {
        inner.runtime.entry(r.id.clone()).or_insert_with(|| ReminderRuntime {
            next_due: now + chrono::Duration::minutes(r.interval_min as i64),
            last_line_index: None,
        });
    }
    inner.reminders = reminders;
}

/// Fires the given reminder (or the water reminder if `reminder_id` is
/// `None`) immediately, bypassing active hours/pause/DND — used by the
/// tray's "Test reminder now" and the Settings "Test" button per reminder.
pub fn trigger_now<R: Runtime>(app: &AppHandle<R>, reminder_id: Option<&str>) {
    let state = app.state::<SchedulerState>();
    let reminder = {
        let inner = state.inner.lock().unwrap();
        match reminder_id {
            Some(id) => inner.reminders.iter().find(|r| r.id == id).cloned(),
            None => inner.reminders.first().cloned(),
        }
    };
    if let Some(reminder) = reminder {
        fire(app, &reminder);
    }
}

pub fn status<R: Runtime>(app: &AppHandle<R>) -> SchedulerStatus {
    let state = app.state::<SchedulerState>();
    let inner = state.inner.lock().unwrap();
    let now = Local::now();

    let active = !inner.dnd && inner.paused_until.map(|u| now >= u).unwrap_or(true);

    let next = inner
        .reminders
        .iter()
        .filter(|r| r.enabled)
        .filter_map(|r| inner.runtime.get(&r.id).map(|rt| (r.id.clone(), rt.next_due)))
        .min_by_key(|(_, due)| *due);

    SchedulerStatus {
        active,
        paused_until: inner.paused_until.map(|t| t.to_rfc3339()),
        next_reminder_id: next.as_ref().map(|(id, _)| id.clone()),
        next_fire_at: next.map(|(_, due)| due.to_rfc3339()),
    }
}
