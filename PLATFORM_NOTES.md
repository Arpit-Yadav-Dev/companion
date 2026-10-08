# Platform notes

Tracks every macOS-specific bit of the implementation and what the Windows
port (Phase 9) will need to do instead. Updated as each phase lands.

## Phase 1: skeleton (tray, settings window, single-instance, activation policy)

- **Dock icon / activation policy** (`src-tauri/src/lib.rs`): we call
  `app.set_activation_policy(tauri::ActivationPolicy::Accessory)` inside
  `#[cfg(target_os = "macos")]`. This is a macOS-only concept — Windows has
  no equivalent Dock, so the Windows build just skips this call. The tray
  icon and taskbar behavior there is controlled entirely by `skip_taskbar`
  on the window builder (set in Phase 2 for the overlay; the settings
  window currently has no taskbar-hiding, matching normal desktop app
  behavior on both OSes).

- **Tray icon style**: `icon_as_template(true)` is passed to
  `TrayIconBuilder` unconditionally. On macOS this makes the icon
  monochrome and theme-adaptive (light/dark menu bar). This flag is a
  no-op on Windows per Tauri's docs, where the tray icon should instead
  ship as a small coloured icon — when we get to Phase 9 we should
  consider swapping the icon asset itself (not just the template flag)
  for a coloured Windows-appropriate version.

- **`tauri-plugin-autostart`** is initialized with
  `MacosLauncher::LaunchAgent`. This enum also has `AppleScript` (macOS-only
  options). On Windows the plugin uses the registry `Run` key internally —
  no code change needed on our side, but it hasn't been tested there yet.

- **Single-instance plugin**: cross-platform as shipped, no macOS-specific
  code. Verified on macOS (launching a second copy exits immediately and
  would focus the existing settings window). Not yet tested on Windows,
  but the plugin's Windows implementation uses a named mutex/IPC pair
  under the hood, same public API.

- **`macOSPrivateApi: true`** in `tauri.conf.json` — required on macOS for
  transparent, undecorated windows (used starting Phase 2 for the
  overlay). This key is ignored on other platforms but is kept
  unconditional in config since Tauri no-ops it rather than erroring.

## Still to do for Windows (tracked here, not implemented yet)

- Verify WebView2 transparent background has no white flash on window show
  (Phase 2).
- Swap tray icon to a coloured asset instead of the template/monochrome
  one used on macOS.
- Test autostart via the registry `Run` key path.
- Test single-instance and window-focus-on-relaunch behavior.
- No GitHub Actions / cross-compiled Windows builds until explicitly
  requested (Phase 9).
