# Companion

> **Work in progress.** Both macOS and Windows build and run. The Windows
> build is much younger and has had far less use, so expect rough edges
> there first.

A tiny menu-bar desktop mascot that reminds you to drink water, stretch,
rest your eyes, and move — by walking onto your screen, asking a quick
yes/no question, and walking back off. Built with Tauri v2 (Rust) +
Svelte 5 + TypeScript.

macOS (Apple Silicon + Intel) and Windows 10/11. macOS came first and is
the more tested of the two; see `PLATFORM_NOTES.md` for what's
platform-specific.

## Features

- A tray/menu-bar icon with live "Next at … · countdown" and
  Active/Paused status
- An animated character that walks onto your screen, asks its question
  with a bold sticker-style speech bubble, and reacts to your answer — all
  while staying click-through everywhere except the character and its
  buttons, so it never gets in your way
- **35 characters.** Fourteen are drawn in SVG and ship with the app
  (4 KB each), so it works offline from first launch. Twenty-one more are
  animated from video and download on demand, 2-7 MB each — that keeps the
  installer at ~7 MB instead of ~75 MB
- Eleven built-in reminders — water, stretch, 20-20-20 eye rest, posture,
  walk, medicine, lunch, breathe, wrap up, time to leave, wind down — plus
  fully custom ones — each with its own
  interval, active hours/days, message lines, button labels, and snooze
  time
- Snooze, pause (30 min / 1 hour / until tomorrow), Do Not Disturb, and
  idle-awareness (pauses while you're away, doesn't nag the moment you
  come back from a long break)
- A Settings window with General, Reminders, Character, Stats, and About
  tabs — reminders and prefs apply live, no restart needed
- A character-pack system: pick any installed character, download more
  from inside Settings, import a `.zip` someone else made, or build your
  own from video clips with the bundled `make-pack` tool
- Fully offline — no accounts, no telemetry, no network calls. All data
  stays in a local JSON store on your machine.

## Running it

```sh
npm install
npm run tauri dev
```

The first Rust build takes a few minutes; subsequent ones are much
faster. This opens the app with a tray icon — look for it in your menu
bar — and no Dock icon (that's intentional; it's an accessory app).

Try the tray's **Test reminder now** to see the full character scene
without waiting for a real reminder to come due.

## Building a release

```sh
npm run tauri build
```

This produces a macOS **universal** build (both Apple Silicon and Intel)
as a `.dmg` under `src-tauri/target/release/bundle/dmg/`, and a `.app`
under `src-tauri/target/release/bundle/macos/`.

**The build is unsigned by default.** That means:
- **macOS** will show a Gatekeeper warning on first launch. Right-click
  the app → **Open** (instead of double-clicking) to get past it — this
  is a one-time thing per machine.
- Proper code signing + notarization requires a paid Apple Developer
  account, which isn't set up here. The build still works fine unsigned;
  it's just an extra click to open it the first time.
- (Windows, when that's built: an unsigned installer will trigger a
  SmartScreen warning the same way.)

## Project structure

```
src-tauri/          Rust backend
  src/
    main.rs          # entry point
    lib.rs            # app setup, tray, window creation, plugin wiring
    overlay.rs         # the transparent click-through overlay window
    scheduler.rs       # when reminders fire — timers, pause, DND, idle-awareness
    reminders.rs       # the Reminder data model + built-in defaults
    stats.rs           # daily glasses/yes/later/missed counts + streaks
    settings.rs        # commands backing the Settings window's 5 tabs
    packs.rs           # character pack discovery, validation, .zip import
    idle.rs            # system idle-time detection (macOS CGEventSource)
    commands.rs        # the rest of the Tauri commands exposed to the frontend
    tray.rs            # tray menu + live status/countdown
  tauri.conf.json
src/                  Svelte frontend
  overlay/             # the overlay window's UI — character, bubble, director
    director.ts         # the scene state machine (walk in -> ask -> react -> walk out)
    Stage.svelte         # ties the director to the character/bubble components
    SvgCharacter.svelte   # "Sip" — the built-in water-droplet character
    ScoutCharacter.svelte # "Scout" — the built-in husky character
    SpritePlayer.ts       # canvas player for imported "frames"-renderer packs
    Bubble.svelte         # the speech bubble (word-pop text, pill buttons)
    Confetti.svelte       # the daily-goal-reached celebration burst
    sound.ts              # entry/yes sound effects
  settings/            # the Settings window's 5 tabs
  shared/types.ts      # typed IPC payloads shared with the Rust side
packs/                 Built-in character packs (pack.json + assets)
tools/make-pack/       Python CLI: turns video clips into a character pack
PLATFORM_NOTES.md       Every macOS-specific piece of code and what Windows will need
```

## Making your own character

See `tools/make-pack/README.md` — it walks through generating a set of
short AI video clips on a green-screen background and running the
`make-pack` tool to turn them into an importable character pack.

## Data & privacy

Everything — reminders, preferences, and stats — is stored locally as
JSON in your app data folder (Settings → About → **Open data folder**).
Nothing is sent anywhere. **Reset all data** in that same tab clears it
back to defaults.

## Removing it

Quit from the tray menu, then delete the app. If you also installed Rust
just for building this (via `rustup`, during setup) and don't need it for
anything else, `rustup self uninstall` removes that too.
