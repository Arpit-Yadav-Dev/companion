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
const WORK_HOURS: ActiveHours = ActiveHours {
    start_minute: 8 * 60,
    end_minute: 22 * 60,
};

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
            prop: None,
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
            prop: None,
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
    ]
}
