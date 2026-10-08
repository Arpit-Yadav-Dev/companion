# Build Prompt: "Companion" — Desktop Mascot Reminder App (Tauri v2, macOS first, Windows later)

> **Instructions for Claude Code:** This file is your full brief. I've already opened this folder in VS Code and started you here. **This folder is the project root.** Build everything inside it (don't create a sub-folder). Start with **Phase 0 (environment setup)**, then go phase by phase. After each phase, stop, tell me how to run and test it, and wait for my OK before continuing.

---

## Phase 0: Environment setup (you do this, safely)

I'm a full-stack web developer and I work on other projects on this Mac. **Setting up this project must not change, upgrade, or break anything else on my machine.** You install what's needed yourself, following these rules strictly.

### Safety rules (non-negotiable)
1. **Check before installing.** For every tool, first check whether it's already installed and which version (`which`, `--version`). Reuse what exists if it meets the minimum version.
2. **Never use `sudo`.** If something truly needs admin rights or a GUI click, stop and give me the exact single command or click to do myself, then continue after I confirm.
3. **Never upgrade, downgrade, or uninstall anything I already have.** That includes my Node version, global npm packages, Homebrew packages, Python, and git config. No `brew upgrade`, no `npm install -g`, no `pip install` outside a project virtualenv, no `nvm alias default`, no changing global git config.
4. **Keep everything project-local where possible:**
   - JS tooling (including `@tauri-apps/cli`) goes in this project's `devDependencies`, run via `npx` / npm scripts.
   - Python tools (for the Phase 7 pack maker) go in a virtualenv at `./tools/.venv`.
   - **ffmpeg comes from a project-local source:** use the `imageio-ffmpeg` package inside `./tools/.venv`, which ships its own ffmpeg binary. **Do not `brew install ffmpeg`.**
   - Rust build output stays in `./src-tauri/target` (default).
5. **The one global install is Rust**, via the official `rustup` installer, which lives entirely in `~/.rustup` and `~/.cargo`. It doesn't touch Node or other tools. Run it as:
   `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal`
   This only adds one line to my shell profile to put `~/.cargo/bin` on PATH. Tell me that it did this. In the current session, run `source "$HOME/.cargo/env"`. If Rust is already installed, don't reinstall it; just run `rustup target add aarch64-apple-darwin x86_64-apple-darwin` (needed for the universal build).
6. **Add a `.gitignore`** covering `node_modules`, `src-tauri/target`, `tools/.venv`, `dist`, and `.DS_Store`.
7. Before running any install command, **print a short plan**: what you'll install, where it goes, and roughly how much disk space it takes. Rust toolchain is about 1–1.5 GB; `target/` grows to several GB. Then proceed.

### Step 0.0: Audit first, install nothing yet
Before installing or creating anything, run a **read-only audit** and show me the results as a short table: tool, installed?, version, path, and action (reuse / install / needs my action).

- **System tools:** Xcode Command Line Tools (`xcode-select -p`), Node and npm (and nvm/fnm/volta if present), Rust (`rustc`, `cargo`, `rustup`, installed targets via `rustup target list --installed`), Python 3, git.
- **This folder:** I've already created a sample app here, so some things may already exist. List what's in the folder: `package.json` and its dependencies, `node_modules`, `src-tauri/` (`Cargo.toml`, `tauri.conf.json`, and the Tauri version: v1 or v2?), frontend framework, `.git`, and any other files.
- **Then decide and tell me:**
  - If it's already a **Tauri v2** app, **reuse it.** Keep what works, adapt it to this brief, and only add what's missing.
  - If it's **Tauri v1**, a different framework, or broken, explain why and propose either migrating it or scaffolding fresh.
  - **Never delete or overwrite my existing files without asking.** If you need to replace things, first move them into `./_backup_<date>/` (and add that folder to `.gitignore`).
- Only install the items marked "install". Skip everything already present and meeting the minimum version.

Wait for my OK on the audit plan, then do the checklist below. Skip any step the audit showed is already done.

### Setup checklist (do in order, report each result)
1. **Xcode Command Line Tools** (needed only for the C linker Rust uses; **not** Xcode.app, which I don't have and won't install): run `xcode-select -p`. If it prints a path, it's installed, so move on. If not, tell me to run `xcode-select --install` and click **Install** in the popup (this is the only GUI step), then wait for me.
2. **Node**: check `node -v`. Need ≥ 18. If I use nvm/fnm/volta, don't switch my default. If the current version is too old, add an `.nvmrc` with a suitable version and tell me to run `nvm use` in this folder only.
3. **Rust**: per rule 5. Verify with `cargo -V` and `rustc -V`.
4. **Only if the audit found no usable Tauri v2 app:** scaffold the Tauri v2 + Svelte + TS project into this current folder using `npm create tauri-app@latest` (non-interactive flags if available; otherwise scaffold to a temp sub-folder and move files up, keeping `CLAUDE.md` intact). Install dependencies locally with `npm install`.
5. **Smoke test**: `npm run tauri dev` should open a window. The first Rust build takes a few minutes; tell me so.
6. Print a final summary of what was installed globally (should be Rust only, plus CLT if it was missing), what is project-local, and how to fully remove everything later: delete this folder, and if I want, run `rustup self uninstall`.

Then stop and wait for my OK before Phase 1.

---

## 0. What we are building (reference behaviour — match this exactly)

A tiny menu-bar / system-tray app. Every N minutes, an animated character **walks onto my screen from the edge**, stands near the bottom-left above the Dock/Taskbar, and **talks to me** with a bold sticker-style speech line and two buttons. It floats above every app, but everything except the character and its buttons is click-through, so I can keep working.

The exact reference sequence (water reminder):

1. Character walks in from the left screen edge (full walk cycle, ~1.2s), stops, turns to face me.
2. Holds up a water bottle. Text pops in above its head, word by word:
   **"Hey, {name}"** / **"Did you drink water?"**
3. Two white pill buttons appear under the text: **[YES]** **[Remind me later]**.
4. Click **YES** → text changes to **"Nice! 💙"**, character drinks from the bottle (celebrate animation), waves, turns and walks off screen. Log one glass.
5. Click **Remind me later** → character shrugs, says something like **"Okay, 10 minutes!"**, walks off. Snooze 10 min.
6. **No response for 25s** → character looks a little sad, says **"I'll come back later…"**, walks off. Log as missed.

Tray/menu-bar menu (from reference):
```
● Active
  Next at 1:11 PM · 1m 42s        (live countdown, disabled item)
  ─────────
  Settings…
  Pause reminders                 (submenu: 30 min / 1 hour / Until tomorrow / Resume)
  Test reminder now
  ─────────
  Quit Companion
```

Visual style of the text: very bold rounded font, **yellow fill (#FFE14D), thick red-orange outline (#E8471C, ~4px stroke via paint-order), soft drop shadow**, slight tilt, each word pops in with a bounce (scale 0.6 → 1.08 → 1, 120ms stagger). Buttons: white pill, dark text, subtle shadow, hover lift, press squish.

The magic is **emotion + motion**: the character must never feel static. Blinking, breathing/idle sway, eye contact, reactions to my choice, and a proper entrance and exit every time.

---

## 1. Hard constraints

**My setup:** I develop on a **Mac**. **Priority right now: make the macOS version perfect and identical in feel to the reference video.** Windows comes later, so don't let it slow down or compromise the Mac experience. That means:
- Mac-first: use macOS-native polish wherever it helps (accessory activation policy, NSWindow collection behaviour, template tray icon, menu-bar countdown).
- Still keep the code **Windows-ready**. Put every OS-specific bit behind `#[cfg(target_os = ...)]` in Rust or a small platform check in TS, and keep each one listed in `PLATFORM_NOTES.md`, with a note on what the Windows version will need. Don't add Windows-only code or testing yet.
- Don't set up GitHub Actions for Windows until I ask (Phase 8 / later). Don't cross-compile Windows locally.

- **Tauri v2** (Rust backend) + **Svelte 5 + TypeScript + Vite** frontend. No Electron, no React (bundle size).
- Targets: **macOS 12+ (Apple Silicon + Intel) now**; **Windows 10/11 later** (code stays ready for it). No iOS. No App Store.
- I do **not** have Xcode and won't install it. Only Xcode **Command Line Tools** are allowed (Phase 0). Never require Xcode.app, never use `xcodebuild`, never open Xcode projects.
- Idle cost must be ~zero: **the overlay window is hidden and the renderer paused between reminders.** Target < 40 MB RAM idle, 0% CPU idle, installer < 15 MB (excluding large character packs).
- Fully offline. No telemetry, no accounts, no network calls.
- All data stored locally via `tauri-plugin-store` (JSON).

Plugins to use: `tauri-plugin-store`, `tauri-plugin-autostart`, `tauri-plugin-single-instance`, `tauri-plugin-notification` (fallback only), `tauri-plugin-dialog` (pack import). Tray via Tauri v2 built-in `tray-icon` feature.

---

## 2. Architecture

```
./   (this folder = project root)
  src-tauri/
    src/
      main.rs            # setup, tray, windows, activation policy
      scheduler.rs       # timers, active hours, snooze, pause, idle-awareness
      overlay.rs         # overlay window: position, show/hide, click-through toggling
      idle.rs            # system idle time (macOS CGEventSource / Windows GetLastInputInfo)
      packs.rs           # character pack discovery, validation, import (zip)
      stats.rs           # daily logs, streaks
      commands.rs        # tauri commands exposed to frontend
    tauri.conf.json
  src/
    overlay/             # overlay window UI (character + bubble)
      Stage.svelte
      SpritePlayer.ts    # canvas frame-sequence player
      SvgCharacter.svelte# built-in code-drawn character
      Bubble.svelte
      director.ts        # the state machine that runs a "scene"
    settings/            # settings window UI
    shared/types.ts
  packs/
    sip/                 # built-in SVG character (default)
  tools/
    make-pack/           # script to turn AI-generated video clips into a pack
```

**Rust owns time. The frontend owns acting.** Rust decides *when* a reminder fires and emits `reminder://show {reminderId, line, options}`. The overlay "director" plays the scene and reports back `reminder_response {reminderId, choice: "yes"|"later"|"skip"|"timeout"}`. Rust logs it and reschedules.

### 2.1 Overlay window (the hard part — do it carefully)

- Created at startup, **hidden**. Config: `transparent: true`, `decorations: false`, `alwaysOnTop: true`, `skipTaskbar: true`, `shadow: false`, `resizable: false`, `focus: false` (must never steal focus from what I'm typing in).
- macOS: set `"macOSPrivateApi": true` in `tauri.conf.json` (required for transparent windows), and `app.set_activation_policy(ActivationPolicy::Accessory)` so there's no Dock icon.
- macOS extra: via `objc2`/`objc2-app-kit`, set the NSWindow collection behaviour to `canJoinAllSpaces | fullScreenAuxiliary | stationary` and level to floating, so the character appears on the current Space and over full-screen apps. Isolate this in `#[cfg(target_os = "macos")]`.
- Size/position: a horizontal strip covering the **work area** (screen minus Dock/Taskbar/menu bar) of the **monitor the cursor is on**, full width, bottom ~55% height. Use the monitor work area API; if unavailable, fall back to full size minus a safe margin and expose a "bottom offset" setting.
- **Click-through with interactive hit regions:** default `set_ignore_cursor_events(true)`. The frontend reports its interactive rects (character body + bubble + buttons) to Rust whenever they change. While the overlay is visible, a Rust loop polls the global cursor position every ~33 ms; when the cursor enters a rect, call `set_ignore_cursor_events(false)`; when it leaves, set it back to `true`. Stop the loop when hidden. This is how clicks pass through to apps behind everything except the character.
- Windows: verify the transparent WebView2 background is truly transparent (no white flash on show). Show the window only after the first frame is painted (frontend emits `overlay_ready`).
- Respect `prefers-reduced-motion`: crossfade in/out instead of walking.

### 2.2 Scheduler

Each reminder has: `id, enabled, emoji, title, interval_min, active_hours (start,end), days_of_week, lines[], yes_label, later_label, snooze_min, prop (e.g. "bottle")`.

Rules:
- Only fire inside active hours / days.
- **Idle-aware:** if system idle > 5 min, pause timers; when I return after > 15 min away, reset the water timer instead of nagging immediately (I wasn't at the desk).
- Never show two reminders at once — queue them, min 90s gap.
- Snooze, pause (30m / 1h / until tomorrow), and "Do not disturb" toggle.
- Smart skip: don't show if the overlay can't be shown sensibly; fall back to a native notification only if the user enables "Notifications fallback".
- Tray "Next at … · countdown" updates every second **only while the menu is open is not detectable**, so update the menu item text every 15s and on every scheduling change (cheap).

Built-in reminders (all editable, all off except water by default):
| Reminder | Default | Example lines |
|---|---|---|
| 💧 Water | 45 min | "Hey, {name} — did you drink water?" / "Sip check! 💧" / "Your brain is 75% water. Top it up?" |
| 🧘 Stretch | 60 min | "Stretch break? Arms up!" |
| 👀 Eye rest (20-20-20) | 20 min | "Look at something far away for 20 seconds 👀" (with a 20s countdown ring in the bubble) |
| 🪑 Posture | 30 min | "Shoulders down, back straight!" |
| 🚶 Walk | 90 min | "Quick walk? Even 2 minutes counts." |
| ✏️ Custom | user | user's own text, emoji, interval, buttons |

Lines support `{name}`, `{count}` (glasses today), `{streak}`. Pick a random line, never the same one twice in a row. Time-of-day flavour: "Good morning, {name}!" before 11am on the first reminder of the day.

### 2.3 Character system ("packs") — this is what makes it special

A **character pack** is a folder with `pack.json` + assets. The app ships with one built-in pack and can import more (`.zip` drag-drop in Settings). Two renderer types:

**Type A — `svg` (built-in, code-drawn, tiny):** a character drawn in SVG and animated with CSS/JS. Ships as the default so the app works on day one.

**Type B — `frames` (realistic / 3D look like the reference video):** each animation state is a sequence of transparent WebP frames played on a `<canvas>` by `SpritePlayer.ts`.

`pack.json`:
```json
{
  "id": "aakash-3d",
  "name": "Aakash (3D)",
  "renderer": "frames",
  "fps": 24,
  "height_px": 420,
  "anchor": { "x": 0.5, "y": 1.0 },
  "bubble_offset": { "x": 0, "y": -0.95 },
  "states": {
    "walk_in":   { "dir": "walk",     "loop": false, "move": "enter" },
    "idle":      { "dir": "idle",     "loop": true },
    "ask":       { "dir": "ask",      "loop": true },
    "happy":     { "dir": "drink",    "loop": false },
    "shrug":     { "dir": "shrug",    "loop": false },
    "sad":       { "dir": "sad",      "loop": false },
    "wave":      { "dir": "wave",     "loop": false },
    "walk_out":  { "dir": "walk",     "loop": false, "move": "exit" }
  },
  "props": { "water": "ask", "stretch": "ask_stretch" },
  "mirror_for_direction": true
}
```

Required states: `walk_in, idle, ask, happy, shrug, sad, wave, walk_out`. Optional per-reminder overrides via `props` (e.g. holding a bottle for water vs. arms up for stretch). Missing optional states fall back to `ask`.

**SpritePlayer performance rules (important for "lightweight"):**
- Decode frames with `createImageBitmap` **only for the current and next state**; `close()` bitmaps of states no longer needed. Never keep a whole pack decoded (300 frames × 420px would be hundreds of MB).
- Use `requestAnimationFrame` with a fixed-fps accumulator; stop the loop entirely when the overlay hides.
- Horizontal movement for `walk_in`/`walk_out` is a CSS transform tween synced to the walk cycle (so the feet don't skate: speed = stride_px × fps / frames_per_stride, configurable in pack.json as `stride_px`).
- Mirror with `scaleX(-1)` when walking right-to-left.

**Built-in `svg` pack "Sip"** — design it to be genuinely charming, not generic:
- A chubby water-droplet character with a glossy highlight, big expressive eyes (separate pupils that track toward the cursor), blush cheeks, tiny stubby arms and feet.
- Procedural animation: breathing squash-and-stretch (2.5s loop), random blinks (every 2–6s, occasional double-blink), eye tracking, bouncy hop-walk for enter/exit with dust puffs.
- Expressions driven by the same states: `ask` (head tilt, eyebrows up, holds mini bottle/prop), `happy` (eyes become ^ ^, jumps, sparkle burst, a few heart particles), `shrug` (arms up, flat mouth, little "eh"), `sad` (droopy eyes, slow walk out, one tiny tear), `wave`.
- Theme variants via CSS variables: Blue (default), Peach, Mint, Night. Also "outfit" accessories per reminder (headband for stretch, glasses for eye-rest).
- Also build a second svg character, **"Pixel"** — a small cat with a 2-colour flat style — to prove the pack system supports multiple characters.

### 2.4 The director (scene state machine)

`director.ts` runs every appearance as:
```
hidden → enter(walk_in) → settle(idle 300ms) → ask(bubble pops in, buttons fade in after text)
  ├─ yes     → happy (+ "Nice! 💙" / random praise) → wave → walk_out → hidden
  ├─ later   → shrug (+ "Okay, {snooze} minutes!") → walk_out → hidden
  ├─ timeout → sad (+ "I'll come back later…") → walk_out → hidden
  └─ hover-character → small reaction (look at cursor / tiny jump) — never blocks the flow
```
- Pick entrance side randomly (left or right) unless "Always from left" is set.
- Clicking the character itself (not the buttons) makes it do a random cute reaction; clicking it 5 times quickly → special easter-egg animation.
- Keyboard: when the overlay is interactive, `Y` = yes, `L` = later, `Esc` = dismiss (counts as later). Never grab global shortcuts.
- Optional sound (off by default): short soft "pop" on entry, "gulp/ding" on yes. Volume slider. Tiny bundled `.ogg`/`.m4a` files.
- Streak moments: on hitting the daily water goal, the character does the happy animation with confetti and says "Goal done! 8/8 🎉".

---

## 3. Settings window

Normal decorated window (~720×520), opened from tray. Tabs:

1. **General** — your name (used in `{name}`), launch at login (autostart), start paused toggle, DND, reduced motion, sound, language placeholder (English only for now).
2. **Reminders** — list of reminders with toggle, interval, active hours, days, lines editor (add/remove/reorder), buttons labels, snooze minutes, "Test" button per reminder. "+ Custom reminder".
3. **Character** — gallery of installed packs with live animated preview, pick active character, theme, size slider (60%–140%), entrance side, position (bottom-left / bottom-right / bottom-center), "Import pack (.zip)".
4. **Stats** — today's glasses vs goal (editable, default 8), 7-day bar chart (drawn with plain SVG, no chart library), current/best streak, responses (yes/later/missed).
5. **About** — version, data folder button, reset all.

Settings changes apply live (no restart).

---

## 4. Pack-making tool (to get the realistic 3D look from the video)

The reference uses a pre-rendered 3D character (looks like an AI-generated Pixar-style avatar of the developer). That cannot be generated by code; it must be produced as video clips and converted. Build `tools/make-pack/` as a Python CLI:

```
./tools/.venv/bin/python tools/make-pack/make_pack.py \
  --name "Aakash (3D)" --id aakash-3d \
  --clip walk=clips/walk.mp4 --clip idle=clips/idle.mp4 --clip ask=clips/ask.mp4 \
  --clip drink=clips/drink.mp4 --clip shrug=clips/shrug.mp4 --clip sad=clips/sad.mp4 \
  --clip wave=clips/wave.mp4 \
  --height 420 --fps 24 --bg auto
```

It must:
1. Extract frames with ffmpeg at the target fps (use the `imageio-ffmpeg` binary from `./tools/.venv`, never a system ffmpeg).
2. Remove background: if clips are on solid green/blue → chroma key (ffmpeg `colorkey` + despill); otherwise use `rembg` (offer `--bg rembg`).
3. Auto-crop all frames of a pack to one shared bounding box (so the character doesn't jitter between states), align feet to the bottom.
4. Resize to `--height`, export WebP (quality 80, alpha), write `pack.json`, zip it.
5. Detect loop seam for looping states (pick the frame most similar to frame 0 as the loop end).
6. Print total size and warn if the pack exceeds 15 MB.

Also write `tools/make-pack/README.md` explaining how I can make the clips: create one consistent character image (e.g. a 3D cartoon version of me, full body, on solid green background), then use any image-to-video AI tool to generate ~2–3s clips for each state, all with locked camera, same framing, same green background, character facing camera (walk clip side-on). Include ready-to-use generation prompts for each state.

---

## 5. Build, CI, distribution

- `npm run tauri dev` for local dev on whatever OS I'm on.
- Now: local `npm run tauri build` producing a macOS universal `.dmg`.
- Later (only when I ask): **GitHub Actions** workflow using `tauri-apps/tauri-action` for Windows `.msi` + `.exe` (x64 + ARM64).
- Code signing is **optional** and off by default. Document clearly: unsigned macOS app shows a Gatekeeper warning (right-click → Open); notarization requires a paid Apple Developer account. Unsigned Windows installer shows SmartScreen. Leave env-var hooks for signing later.
- App identifier: `com.companion.app` (I'll change later). Tray icon: monochrome template icon on macOS (`icon_as_template(true)`), coloured on Windows.

---

## 6. Phases (do them in order, stop after each)

0. **Environment setup** (see Phase 0 at the top).
1. **Skeleton:** Tauri v2 + Svelte project, tray with full menu (static), settings window opens, single-instance, macOS accessory policy, no Dock icon.
2. **Overlay:** transparent always-on-top click-through strip on the cursor's monitor, hit-region toggling, show/hide, "Test reminder now" shows a placeholder box with working YES / Later buttons. Verify on both OSes that clicks pass through elsewhere and no focus is stolen.
3. **Built-in "Sip" character + director:** full scene flow (enter → ask → yes/later/timeout → exit), bubble styling exactly as section 0, word-pop animation, blinking, breathing, eye tracking.
4. **Scheduler + store:** reminders, intervals, active hours, snooze, pause, DND, idle awareness, live "Next at" in tray.
5. **Settings UI + stats** (all 5 tabs).
6. **Frames renderer + pack system + import**, second svg character "Pixel".
7. **make-pack tool + README with clip-generation prompts.**
8. **Polish:** sounds, reduced-motion, multi-monitor, autostart, easter egg, confetti, `.dmg` build, README for the whole app.
9. **Windows (later, only when I say go):** CI workflow, Windows-specific fixes from `PLATFORM_NOTES.md`, VM test checklist.

For each phase: list files changed, how to run, what to manually test on macOS, and anything that will need attention for Windows later (add it to `PLATFORM_NOTES.md`). If a Tauri v2 API you plan to use has a different name or doesn't exist in the installed version, check `cargo doc`/the crate source instead of guessing, and tell me what you changed.

## 7. Quality bar

- Animations at a steady 60fps for SVG, pack fps for frames; no layout jank when the bubble appears.
- No white flash, no focus steal, no blocking clicks outside the character — these three are bugs, not polish.
- Idle: overlay hidden, render loops stopped, cursor polling stopped.
- Code: small modules, typed IPC payloads shared between Rust (serde) and TS, no `unwrap()` on user data, errors logged to a rotating log file in the app data dir.
