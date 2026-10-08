//! System idle-time detection, used by the scheduler to pause timers while
//! the user is away and to decide whether a long absence should reset
//! (rather than immediately re-fire) a reminder.

/// Returns how long the system has been idle (no keyboard/mouse/tablet
/// input), or `None` if idle time couldn't be determined on this platform.
#[cfg(target_os = "macos")]
pub fn idle_duration() -> Option<std::time::Duration> {
    mac::seconds_since_last_input().map(std::time::Duration::from_secs_f64)
}

#[cfg(not(target_os = "macos"))]
pub fn idle_duration() -> Option<std::time::Duration> {
    // Windows implementation (Phase 9): GetLastInputInfo from user32, then
    // GetTickCount() - lastInputInfo.dwTime, converted to a Duration. See
    // PLATFORM_NOTES.md.
    None
}

#[cfg(target_os = "macos")]
mod mac {
    /// `kCGAnyInputEventType` from CGEventSource.h — passing this covers
    /// idle time across any keyboard/mouse/tablet input, not just one
    /// event type. Value confirmed from the actual SDK header shipped
    /// with Xcode Command Line Tools:
    /// `CoreGraphics.framework/Headers/CGEventTypes.h`:
    /// `#define kCGAnyInputEventType ((CGEventType)(~0))`
    const K_CG_ANY_INPUT_EVENT_TYPE: u32 = u32::MAX;

    /// `kCGEventSourceStateHIDSystemState` — reflects real hardware input,
    /// as opposed to `CombinedSessionState` (includes events posted by
    /// other processes) or `Private`.
    const K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE: i32 = 1;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(
            state_id: i32,
            event_type: u32,
        ) -> f64;
    }

    pub fn seconds_since_last_input() -> Option<f64> {
        let seconds = unsafe {
            CGEventSourceSecondsSinceLastEventType(
                K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE,
                K_CG_ANY_INPUT_EVENT_TYPE,
            )
        };
        if seconds.is_finite() && seconds >= 0.0 {
            Some(seconds)
        } else {
            None
        }
    }
}
