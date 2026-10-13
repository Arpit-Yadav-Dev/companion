//! Reminder configuration: the data model and built-in defaults from the
//! brief's table. Persisted under the `"reminders"` key in the main store.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveHours {
    pub start_minute: u32,
    pub end_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub enabled: bool,
    pub emoji: String,
    pub title: String,
    pub interval_min: u32,
    pub active_hours: ActiveHours,
    pub days_of_week: Vec<u8>, // 0 = Sunday .. 6 = Saturday
    pub lines: Vec<String>,
    pub yes_label: String,
    pub later_label: String,
    pub snooze_min: u32,
    pub prop: Option<String>,
}

const ALL_DAYS: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
const WEEKDAYS: [u8; 5] = [1, 2, 3, 4, 5];
const WORK_HOURS: ActiveHours = ActiveHours {
    start_minute: 8 * 60,
    end_minute: 22 * 60,
};

/// A once-a-day reminder is expressed as a long interval inside a narrow
/// active window: the scheduler fires the first time the window opens and
/// the interval then carries past the window's end, so it can't repeat.
const fn once_daily_at(hour: u32, minute: u32) -> ActiveHours {
    ActiveHours {
        start_minute: hour * 60 + minute,
        end_minute: hour * 60 + minute + 45,
    }
}
const DAILY_INTERVAL_MIN: u32 = 12 * 60;

/// The built-in reminders from the brief's table — all editable, all off
/// by default except water.
pub fn built_in_defaults() -> Vec<Reminder> {
    vec![
        Reminder {
            id: "water".into(),
            enabled: true,
            emoji: "💧".into(),
            title: "Water".into(),
            interval_min: 45,
            active_hours: WORK_HOURS,
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec![
                "Hey, {name} — did you drink water?".into(),
                "Sip check! 💧".into(),
                "Your brain is 75% water. Top it up?".into(),
            ],
            yes_label: "YES".into(),
            later_label: "Remind me later".into(),
            snooze_min: 10,
            prop: Some("bottle".into()),
        },
        Reminder {
            id: "stretch".into(),
            enabled: false,
            emoji: "🧘".into(),
            title: "Stretch".into(),
            interval_min: 60,
            active_hours: WORK_HOURS,
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec!["Stretch break? Arms up!".into()],
            yes_label: "Done!".into(),
            later_label: "Remind me later".into(),
            snooze_min: 10,
            prop: Some("stretch".into()),
        },
        Reminder {
            id: "eye_rest".into(),
            enabled: false,
            emoji: "👀".into(),
            title: "Eye rest (20-20-20)".into(),
            interval_min: 20,
            active_hours: WORK_HOURS,
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec!["Look at something far away for 20 seconds 👀".into()],
            yes_label: "Done!".into(),
            later_label: "Remind me later".into(),
            snooze_min: 5,
            prop: Some("lookfar".into()),
        },
        Reminder {
            id: "posture".into(),
            enabled: false,
            emoji: "🪑".into(),
            title: "Posture".into(),
            interval_min: 30,
            active_hours: WORK_HOURS,
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec!["Shoulders down, back straight!".into()],
            yes_label: "Fixed!".into(),
            later_label: "Remind me later".into(),
            snooze_min: 10,
            prop: Some("posture".into()),
        },
        Reminder {
            id: "walk".into(),
            enabled: false,
            emoji: "🚶".into(),
            title: "Walk".into(),
            interval_min: 90,
            active_hours: WORK_HOURS,
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec!["Quick walk? Even 2 minutes counts.".into()],
            yes_label: "Done!".into(),
            later_label: "Remind me later".into(),
            snooze_min: 15,
            prop: None,
        },
        // Everyday reminders. The ones above repeat through the day to
        // interrupt a habit; these happen once, at a time that matters, so
        // they use a narrow window and a long interval instead.
        Reminder {
            id: "medicine".into(),
            enabled: false,
            emoji: "💊".into(),
            title: "Medicine".into(),
            interval_min: DAILY_INTERVAL_MIN,
            active_hours: once_daily_at(9, 0),
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec![
                "Time for your medicine, {name}.".into(),
                "Don't forget your medicine 💊".into(),
            ],
            yes_label: "Taken".into(),
            later_label: "In a bit".into(),
            snooze_min: 15,
            prop: None,
        },
        Reminder {
            id: "lunch".into(),
            enabled: false,
            emoji: "🍜".into(),
            title: "Lunch break".into(),
            interval_min: DAILY_INTERVAL_MIN,
            active_hours: once_daily_at(13, 0),
            days_of_week: WEEKDAYS.to_vec(),
            lines: vec![
                "Lunch time, {name}. Step away from the screen.".into(),
                "Food? Proper food, not another coffee ☕".into(),
            ],
            yes_label: "Going now".into(),
            later_label: "Ten more minutes".into(),
            snooze_min: 10,
            prop: None,
        },
        Reminder {
            id: "eyes_break".into(),
            enabled: false,
            emoji: "😮‍💨".into(),
            title: "Breathe".into(),
            interval_min: 120,
            active_hours: WORK_HOURS,
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec![
                "Three slow breaths. In through the nose.".into(),
                "Unclench your jaw. Drop your shoulders.".into(),
            ],
            yes_label: "Done".into(),
            later_label: "Not now".into(),
            snooze_min: 20,
            prop: None,
        },
        Reminder {
            id: "wrap_up".into(),
            enabled: false,
            emoji: "📝".into(),
            title: "Wrap up".into(),
            interval_min: DAILY_INTERVAL_MIN,
            active_hours: once_daily_at(17, 30),
            days_of_week: WEEKDAYS.to_vec(),
            lines: vec![
                "Half an hour left. Start wrapping up?".into(),
                "Good time to note where you left off.".into(),
            ],
            yes_label: "On it".into(),
            later_label: "Still working".into(),
            snooze_min: 15,
            prop: None,
        },
        Reminder {
            id: "go_home".into(),
            enabled: false,
            emoji: "🏠".into(),
            title: "Time to leave".into(),
            interval_min: DAILY_INTERVAL_MIN,
            active_hours: once_daily_at(18, 30),
            days_of_week: WEEKDAYS.to_vec(),
            lines: vec![
                "That's a day, {name}. Go home.".into(),
                "Work will still be here tomorrow 🏠".into(),
            ],
            yes_label: "Heading out".into(),
            later_label: "One more thing".into(),
            snooze_min: 20,
            prop: None,
        },
        Reminder {
            id: "sleep".into(),
            enabled: false,
            emoji: "🌙".into(),
            title: "Wind down".into(),
            interval_min: DAILY_INTERVAL_MIN,
            active_hours: once_daily_at(22, 30),
            days_of_week: ALL_DAYS.to_vec(),
            lines: vec![
                "Screens off soon? Sleep makes tomorrow easier.".into(),
                "Time to wind down, {name} 🌙".into(),
            ],
            yes_label: "Logging off".into(),
            later_label: "Not yet".into(),
            snooze_min: 20,
            prop: None,
        },
    ]
}
