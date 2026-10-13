//! System idle-time detection, used by the scheduler to pause timers while
//! the user is away and to decide whether a long absence should reset
//! (rather than immediately re-fire) a reminder.

/// Returns how long the system has been idle (no keyboard/mouse/tablet
/// input), or `None` if idle time couldn't be determined on this platform.
#[cfg(target_os = "macos")]
pub fn idle_duration() -> Option<std::time::Duration> {
    mac::seconds_since_last_input().map(std::time::Duration::from_secs_f64)
}

#[cfg(target_os = "windows")]
pub fn idle_duration() -> Option<std::time::Duration> {
    win::seconds_since_last_input().map(std::time::Duration::from_secs_f64)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn idle_duration() -> Option<std::time::Duration> {
    None
}

/// Whether the main display is asleep (lid shut, screen off, or the
/// screen saver has blanked it).
///
/// Idle time alone doesn't cover this: the display can sleep while the
/// machine stays awake, and a reminder that fires then is spent on a dark
/// screen — the user never sees it, but it still counts as missed and
/// still resets the timer.
#[cfg(target_os = "macos")]
pub fn display_is_asleep() -> bool {
    mac::display_is_asleep()
}

#[cfg(target_os = "windows")]
pub fn display_is_asleep() -> bool {
    win::display_is_off()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn display_is_asleep() -> bool {
    false
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

        /// `CGMainDisplayID()` — the display the menu bar is on.
        fn CGMainDisplayID() -> u32;
        /// `CGDisplayIsAsleep()` — true while that display is powered down,
        /// including lid-closed and screen-saver blanking.
        fn CGDisplayIsAsleep(display: u32) -> i32;
    }

    pub fn display_is_asleep() -> bool {
        unsafe { CGDisplayIsAsleep(CGMainDisplayID()) != 0 }
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


#[cfg(target_os = "windows")]
mod win {
    use windows_sys::Win32::System::Power::{
        GetSystemPowerStatus, SYSTEM_POWER_STATUS,
    };
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    pub fn seconds_since_last_input() -> Option<f64> {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        // SAFETY: cbSize is set to the struct's own size, as the API requires.
        if unsafe { GetLastInputInfo(&mut info) } == 0 {
            return None;
        }
        // dwTime is a 32-bit tick count that wraps every ~49 days, while
        // GetTickCount64 doesn't. Compare in 32-bit space so a wrap between
        // the two reads cancels out instead of yielding a wild number.
        let now = unsafe { GetTickCount64() } as u32;
        let elapsed_ms = now.wrapping_sub(info.dwTime);
        Some(elapsed_ms as f64 / 1000.0)
    }

    /// Whether the display is powered down.
    ///
    /// Windows has no direct "is the display asleep" query — the documented
    /// route is to register for GUID_CONSOLE_DISPLAY_STATE notifications,
    /// which needs a window to receive them. This uses the one pollable
    /// signal available instead: the system power status reports the display
    /// as off while it's blanked.
    pub fn display_is_off() -> bool {
        let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
        // SAFETY: the struct is zeroed and matches the API's expected layout.
        if unsafe { GetSystemPowerStatus(&mut status) } == 0 {
            return false;
        }
        // 0 = off, 1 = on, 255 = unknown. Only treat a definite "off" as
        // asleep, so an unknown value never suppresses reminders.
        status.SystemStatusFlag == 0 && status.ACLineStatus != 255
    }
}
