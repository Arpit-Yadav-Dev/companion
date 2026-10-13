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

## Phase 3: "Sip" character + director

- **Eye-tracking gotcha (important, cross-platform-relevant):** the
  character's pupils originally tracked the cursor via a DOM `mousemove`
  listener (`window.addEventListener("mousemove", ...)`). This only ever
  fired once the cursor had already been hovered/clicked on the character
  at least once, because the overlay window is click-through
  (`set_ignore_cursor_events(true)`) everywhere except directly over an
  active hit rect — and a window that's ignoring cursor events does not
  receive native mouse events from the OS at all outside that rect. Fixed
  by having the existing Rust-side hit-region polling thread
  (`overlay.rs::start_hit_region_polling`, already sampling the global
  cursor at ~33ms) also broadcast the cursor's position, converted into
  the overlay's own logical coordinate space, on a new `overlay://cursor`
  event; `SvgCharacter.svelte` listens for that instead of DOM
  `mousemove`. This is pure Tauri API (`emit_to` / `WebviewWindow::listen`)
  with no macOS-specific pieces, so it should work unmodified on Windows
  — but it's worth a deliberate check there too, since Win32's own
  equivalent of "a window with click-through set doesn't receive mouse
  events" needs to hold for the same reasoning to apply.

- **Mirrored-pupil bug**: when the character enters from the right side of
  the screen, `Stage.svelte` mirrors it with `scaleX(-1)` on its wrapper
  div so it visually faces the direction of travel. `getBoundingClientRect()`
  on the character's root element already reflects that mirrored,
  post-transform screen position correctly, but the *offset* computed
  from cursor-to-character-center needs negating on the x-axis before
  being applied inside the (also mirrored) SVG's own coordinate space —
  otherwise the pupils visibly move opposite to the cursor. Fixed by
  passing a `mirrored` prop down from `Stage.svelte` (which already knows
  `entranceSide`) into `SvgCharacter.svelte`. Pure CSS/math, no
  platform-specific code.

## Phase 4: scheduler + store

- **Idle detection** (`src-tauri/src/idle.rs`) is the one genuinely
  OS-specific piece this phase. macOS has no safe-rust crate already in
  our dependency tree exposing `CGEventSourceSecondsSinceLastEventType`,
  so it's a small hand-written `extern "C"` binding against the
  already-linked `CoreGraphics` framework (linked transitively via
  `core-graphics`, a dependency of `tauri`/`objc2-core-graphics`, so this
  adds no new linker step). Two constants had to be sourced carefully
  rather than guessed:
  - `kCGAnyInputEventType` is `0xFFFFFFFF` (`u32::MAX`), confirmed by
    reading the actual SDK header on this machine
    (`CoreGraphics.framework/Headers/CGEventTypes.h`:
    `#define kCGAnyInputEventType ((CGEventType)(~0))`) — *not* `0`
    (`kCGEventNull`, a different, specific event type), which would have
    silently measured "seconds since last null event" instead of idle
    time.
  - `kCGEventSourceStateHIDSystemState` = `1`, for real hardware input as
    opposed to events posted by other processes.
  Entirely isolated in `#[cfg(target_os = "macos")]`; the `#[cfg(not(...))]`
  fallback returns `None` (treated as "not idle" by the scheduler) until
  Phase 9 implements the Windows path via `GetLastInputInfo` from
  `user32.dll` (see the stub's doc comment in `idle.rs` for the exact
  approach: `GetTickCount() - lastInputInfo.dwTime`).

- **Scheduler tick loop, store, stats** (`scheduler.rs`, `reminders.rs`,
  `stats.rs`): pure Tauri/tokio/chrono, no OS-specific code — the 1Hz tick
  loop, active-hours/day-of-week checks, snooze/pause/DND, and the daily
  stats log all use portable APIs and should need zero changes for
  Windows.

- **Live tray countdown** (`tray.rs`): `MenuItem::set_text()` + a 15s
  ticker, cross-platform Tauri API. No platform branch, but worth a visual
  check on Windows that the tray menu item text updates the same way (no
  known reason it wouldn't).

## Phase 6: frames renderer + pack system + import

- **Bundled `packs/` resolution** (`packs.rs::builtin_packs_dir`): resources
  declared in `tauri.conf.json`'s `bundle.resources` (using the `{"from/":
  "to/"}` map form, not a bare string — a bare `"../packs"` gets its `..`
  rewritten to a literal `_up_` path segment by Tauri, landing the folder
  at the wrong place) are copied into `target/<profile>/` on every build,
  including `tauri dev`'s. This made `resource_dir()` resolve correctly
  in dev without needing a project-relative fallback, which surprised me
  enough to verify against the installed `tauri-build`/`tauri-utils`
  source rather than assume — worth remembering if a future Tauri
  version changes this. A `CARGO_MANIFEST_DIR`-relative fallback is kept
  anyway for the narrow gap before the first resource copy has happened.
  Cross-platform — no code changes expected for Windows, but worth a
  sanity check once there's a Windows build, since resource-dir
  resolution differs structurally there (no `.app/Contents/Resources`).

- **Pack import** (`packs.rs::import_pack`): uses `tauri-plugin-dialog`'s
  native file picker and the `zip` crate's `ZipArchive::extract`, both
  cross-platform. No OS-specific code. The native file-picker UI will
  obviously look like Windows' own picker there rather than macOS's, but
  that's just the platform being itself, not something we control or
  need to branch on.

- **`SpritePlayer.ts`** (frame-sequence canvas player for `frames`-
  renderer packs): pure web APIs (`createImageBitmap`, `requestAnimationFrame`,
  `fetch`), nothing macOS-specific. Not yet exercised end-to-end since no
  real `frames` pack exists until Phase 7's make-pack tool produces one —
  worth a dedicated test pass once that content exists, on both
  platforms.

## Phase 7: make-pack tool

Pure Python + ffmpeg (via `imageio-ffmpeg`'s bundled binary, never
system/Homebrew ffmpeg per the Phase 0 rules) + Pillow/numpy — nothing
macOS-specific. Verified end-to-end against synthetic green-screen test
clips: frame extraction, chroma-key + despill, shared-bbox cropping,
WebP export with alpha (confirmed alpha only survives when the source
actually has soft/anti-aliased edges — a hard-edged test shape that
crops tight to a fully-opaque region legitimately encodes without an
alpha channel, which is correct WebP-encoder behavior, not a bug), loop
seam detection, and zip packaging all worked. Should run identically on
Windows — `imageio-ffmpeg` ships Windows binaries too, and nothing else
here touches the OS. Not yet tested on a Windows machine.

## Phase 8: polish

- **Sound** (`src/overlay/sound.ts`): Web Audio API (`AudioContext`,
  oscillators), pure web standard, nothing macOS-specific. **These are
  placeholder synthesized tones, not real recorded sound-design assets**
  — the brief calls for bundled `.ogg`/`.m4a` files, and that swap is a
  tracked follow-up (see the `sound-assets-followup` memory), not
  something done yet.

- **Reduced motion**: now driven two ways — the OS-level
  `prefers-reduced-motion` media query (already there from Phase 2) and
  the app's own `GeneralPrefs.reducedMotion` setting, which is read fresh
  on every overlay appearance and applied both to the CSS (via a
  `.reduced-motion` class on `.scene` in `Stage.svelte`, mirroring the
  media query) and to the director's own wait timings (`REDUCED_WALK_MS`
  in `director.ts`, so the shortened CSS crossfade and the state-machine
  timing stay in sync instead of the state jumping ahead of the
  animation). Pure CSS/JS, no platform branch needed.

- **Confetti / easter egg**: both pure frontend (Svelte component +
  click-counting), no OS-specific code.

- **`reminder_response` / `end_scene` split**: previously the frontend
  called a single `reminder_response` command only after the entire
  walk-out animation finished, which meant stats were recorded late and
  there was no way to show a goal-reached moment while the character was
  still visible. Split into `reminder_response` (called immediately on
  choice; logs stats and returns them so the frontend can detect
  "just hit today's goal") and `end_scene` (called after the walk-out
  animation completes; actually hides the window). Pure architecture
  fix, no platform-specific code — but worth remembering if Windows
  Phase 9 testing ever needs to trace an overlay-hide-timing bug, since
  this is exactly the kind of ordering issue that caused the Phase 2
  "overlay never shows" bug.

- **A real, if minor, bug found and fixed this phase**: `scheduler.rs`'s
  `load_settings` was reading `user_name`/`dnd`/`start_paused` from
  non-existent top-level store keys (`"userName"`, etc.), while
  `settings.rs`'s `save_general_prefs` — the only writer for these
  values — stores them nested under a single `"general"` object. This
  meant the scheduler's copy of the user's name, DND state, and
  start-paused preference were silently wrong on every fresh app launch
  (only correct after the first in-session Settings edit, since
  `save_general_prefs` also pushes the value into the live
  `SchedulerState` directly). Fixed by reading from the correct nested
  path, and by actually applying `start_paused`/`dnd` at startup (neither
  was wired to the scheduler's initial state before this).

- **Release build verified**: `npm run tauri build -- --target
  universal-apple-darwin` successfully produced a genuine universal
  binary (`lipo -info` confirms both `x86_64` and `arm64` slices), a
  10.68 MiB `.app`, and a 4.76 MiB `.dmg` — well under the 15 MB
  installer target from the brief. Bundled `packs/` resources landed at
  the correct `Contents/Resources/packs/` path inside the `.app` (the
  `{"../packs/": "packs/"}` map-form resource config from Phase 6 avoids
  the `_up_` path-mangling a bare `"../packs"` string would have caused).
  This is unsigned, so first launch needs right-click → Open to bypass
  Gatekeeper (documented in the project README). Not yet attempted on
  Windows — that's Phase 9, gated on being asked to start it.

## Post-Phase-8: more characters, visual polish, frames renderer wiring

- **Four new built-in SVG characters** (Biscuit the dog, Sprout the potted
  plant, Sunny the sun, Marshmallow the floaty ghost): same pattern as
  Sip/Pixel, pure CSS/SVG/cursor-broadcast eye-tracking, no platform-
  specific code.

- **Settings UI visual polish**: a shared `theme.css` token system (colour
  tokens, spacing scale, card/button primitives) applied across all 5
  tabs, plus a redesigned Character tab gallery using CSS grid
  (`repeat(auto-fill, minmax(...))`) instead of a flex row, which is what
  was actually overflowing once there were 6+ characters to show. Pure
  frontend, no platform-specific code.

- **Frames renderer finally wired up** (it existed since Phase 6 as
  `SpritePlayer.ts` but was never connected to anything): added
  `FramesCharacter.svelte` as the Svelte wrapper, and enabled Tauri's
  `asset://` protocol to let the frontend fetch WebP frame files from
  disk:
  - `Cargo.toml`: added the `protocol-asset` feature to the `tauri` dep.
  - `tauri.conf.json`: `app.security.assetProtocol.enable: true` with an
    empty static `scope` — the real scope is built at runtime instead
    (see next point), since pack directories live in paths only knowable
    at runtime (bundled resource dir in a packaged build, differs between
    dev/.app; the user's per-OS app-data dir for imported packs).
  - `packs.rs::list_packs`: calls `app.asset_protocol_scope().allow_directory(...)`
    for both the bundled and user packs dirs every time packs are listed
    (idempotent, cheap) — this is the piece that makes the asset
    protocol actually able to serve frame images from wherever packs
    happen to live on this particular install.
  - Cross-platform in principle (`asset://` works the same way on
    Windows), but genuinely untested there — Phase 9 should verify the
    asset-protocol scope setup and the `convertFileSrc()` URL format both
    behave the same way on Windows' webview.

- **Tested against a real (non-AI-generated) video clip** for the first
  time: a stock "dancing dog" clip on a solid blue background (not green)
  proved `make_pack.py`'s chroma-key step needed to support an arbitrary
  key colour, not just green — added `--colorkey-hex` and `--despill`
  {green,blue} flags. Found and left as a known issue (not yet fixed):
  despill doesn't fully neutralize a blue-screen colour cast on
  very light-coloured subjects (faint blue tint remained on the dog's
  light fur near edges) — worth a better despill pass or a stronger
  `colorkey` similarity/blend tuning if a real light-coloured character
  pack ever needs a blue (rather than green) background.

- **Known rough edges in the first frames-renderer integration, to fix
  before this is "real"**: the user reported the character renders
  partially cut off / mispositioned, and the YES/Later buttons become
  unclickable when a frames-pack character is active. Suspect causes to
  investigate: `FramesCharacter`'s canvas sizing (`displayWidth`/
  `displayHeight` placeholder-aspect guess before the first real frame
  loads) likely doesn't match the actual cropped-frame aspect ratio from
  `make_pack.py`, and/or hit-region reporting in `Stage.svelte` reads
  `characterEl`'s bounding box before the canvas has resized to its real
  content, leaving stale (and probably wrong) click-through hit rects.
  Not yet debugged — picking this up again means tracing actual rect
  values the same way the Phase 2 and Phase 8 bugs were traced (eprintln
  tracing through a temporary `debug_*` command, removed again once
  fixed).

## Still to do for Windows (tracked here, not implemented yet)

- Verify WebView2 transparent background has no white flash on window show
  (Phase 2).
- Swap tray icon to a coloured asset instead of the template/monochrome
  one used on macOS.
- Test autostart via the registry `Run` key path.
- Test single-instance and window-focus-on-relaunch behavior.
- Implement `idle::idle_duration()` for Windows via `GetLastInputInfo`
  (user32.dll) — see the doc comment in `idle.rs`'s `#[cfg(not(target_os
  = "macos"))]` stub.
- Implement `idle::display_is_asleep()` for Windows. macOS uses
  `CGDisplayIsAsleep(CGMainDisplayID())`; Windows has no equivalent poll,
  so this needs `RegisterPowerSettingNotification` with
  `GUID_CONSOLE_DISPLAY_STATE` and a hidden message window caching the
  last `PBT_POWERSETTINGCHANGE` state. Until then the stub returns false,
  which only costs the display-asleep optimisation — the sleep-gap
  detection in `scheduler.rs` is platform-independent and still works.
- No GitHub Actions / cross-compiled Windows builds until explicitly
  requested (Phase 9).
