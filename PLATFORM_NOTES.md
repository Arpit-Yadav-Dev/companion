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

## Phase 2: overlay window (transparent, click-through, hit regions)

- **`cursor_position()`** (used in `overlay.rs` both for picking the
  monitor to show on and in the 33ms hit-region poll) returns global
  desktop coordinates identically on macOS and Windows per Tauri's docs —
  no platform branch needed here.

- **Monitor work-area / `monitor_from_point`**: cross-platform Tauri API,
  but the underlying work-area calculation differs — macOS subtracts the
  menu bar and Dock, Windows subtracts the taskbar. No code change needed,
  just something to visually confirm once we have a Windows machine: the
  bottom-anchored strip should sit just above the taskbar there the same
  way it sits above the Dock here.

- **`macOSPrivateApi: true`** (set in Phase 1) is what makes
  `transparent: true` actually produce a transparent NSWindow instead of a
  black one. On Windows, WebView2's transparency works differently (via
  `set_background_color` on the Webview2Controller, which Tauri handles
  internally) — nothing extra needed on our side, but Phase 9 should do a
  visual check for a white/black flash on first show, since WebView2 has a
  known history of flashing opaque before its first transparent paint. Our
  `overlay_ready` → `overlay::show` sequencing (window created `visible:
  false`, frontend tells Rust to show only after its first real frame is
  painted) is designed to dodge this on both platforms, but it's only been
  verified on macOS so far.

- **Gotcha found during Phase 2 testing (WKWebView-specific, may or may
  not reproduce on WebView2):** the original overlay code awaited
  `requestAnimationFrame` *before* calling `invoke("overlay_ready")` (i.e.
  before telling Rust to actually show the window). On macOS, WKWebView
  never fires `requestAnimationFrame` callbacks while its window is
  hidden, so that `await` hung forever and the reminder box never
  appeared — clicking "Test reminder now" did nothing visible, even
  though the Rust-side emit and the frontend's event listener were both
  working correctly. Fixed in `src/routes/overlay/+page.svelte` by
  calling `overlay_ready` first (which shows the window) and only then
  awaiting the next animation frame before measuring hit regions. Windows'
  WebView2 may or may not suspend rAF on hidden windows the same way —
  worth a deliberate regression check in Phase 9, but the current
  show-before-rAF ordering is correct regardless of platform.

- **NSWindow collection behaviour / level** (canJoinAllSpaces,
  fullScreenAuxiliary, stationary, floating) from the brief's section 2.1
  is **not yet implemented** — Phase 2 only uses the portable
  `always_on_top(true)` builder flag, which keeps the overlay above normal
  windows but does not yet guarantee it floats over full-screen apps or
  follows the user across Spaces. This objc2/objc2-app-kit layer is
  deferred to Phase 3 (once the overlay has real content worth seeing
  across Spaces) and will be isolated in `#[cfg(target_os = "macos")]`
  inside `overlay.rs`. No Windows equivalent needed — Win32 topmost windows
  don't have a "Spaces" concept.

- **Hit-region polling thread** (`overlay.rs::start_hit_region_polling`):
  pure Rust using `WebviewWindow::cursor_position` / `set_ignore_cursor_events`,
  no OS-specific code. Confirmed working on macOS end-to-end (cargo check
  clean, app launches, overlay window created hidden with no Dock/taskbar
  entry). Same code path should work unmodified on Windows.

## Still to do for Windows (tracked here, not implemented yet)

- Verify WebView2 transparent background has no white flash on window show
  (Phase 2).
- Swap tray icon to a coloured asset instead of the template/monochrome
  one used on macOS.
- Test autostart via the registry `Run` key path.
- Test single-instance and window-focus-on-relaunch behavior.
- No GitHub Actions / cross-compiled Windows builds until explicitly
  requested (Phase 9).
