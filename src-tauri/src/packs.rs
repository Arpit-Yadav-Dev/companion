//! Character pack discovery, validation, and import. A pack is a folder
//! with `pack.json` + assets, living either in the bundled `packs/`
//! directory (shipped with the app) or the user packs directory under the
//! app data dir (imported via .zip).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::DialogExt;

pub const REQUIRED_STATES: &[&str] = &[
    "walk_in", "idle", "ask", "happy", "shrug", "sad", "wave", "walk_out",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateConfig {
    pub dir: String,
    #[serde(rename = "loop")]
    pub is_loop: bool,
    #[serde(rename = "move", skip_serializing_if = "Option::is_none")]
    pub move_kind: Option<String>, // "enter" | "exit"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub id: String,
    pub name: String,
    pub renderer: String, // "svg" | "frames"
    #[serde(default = "default_fps")]
    pub fps: u32,
    #[serde(default = "default_height")]
    pub height_px: u32,
    #[serde(default = "default_anchor")]
    pub anchor: AnchorPoint,
    #[serde(default = "default_bubble_offset")]
    pub bubble_offset: AnchorPoint,
    #[serde(default)]
    pub states: std::collections::HashMap<String, StateConfig>,
    #[serde(default)]
    pub props: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub mirror_for_direction: bool,
    #[serde(default)]
    pub stride_px: Option<f64>,
}

fn default_fps() -> u32 {
    24
}
fn default_height() -> u32 {
    420
}
fn default_anchor() -> AnchorPoint {
    AnchorPoint { x: 0.5, y: 1.0 }
}
fn default_bubble_offset() -> AnchorPoint {
    AnchorPoint { x: 0.0, y: -0.95 }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub manifest: PackManifest,
    /// Absolute path to the pack's directory, so the frontend can build
    /// asset:// URLs for frame images if this is a `frames` pack.
    pub dir: String,
}

fn validate_manifest(manifest: &PackManifest) -> Result<(), String> {
    if manifest.id.trim().is_empty() {
        return Err("pack.json: \"id\" must not be empty".into());
    }
    if manifest.name.trim().is_empty() {
        return Err("pack.json: \"name\" must not be empty".into());
    }
    if manifest.renderer != "svg" && manifest.renderer != "frames" {
        return Err(format!(
            "pack.json: \"renderer\" must be \"svg\" or \"frames\", got \"{}\"",
            manifest.renderer
        ));
    }
    if manifest.renderer == "frames" {
        for required in REQUIRED_STATES {
            if !manifest.states.contains_key(*required) {
                return Err(format!(
                    "pack.json: frames pack is missing required state \"{required}\""
                ));
            }
        }
    }
    Ok(())
}

/// Resolves the bundled `packs/` directory. `tauri-build` copies whatever
/// `bundle.resources` declares into `target/<profile>/` on every
/// `cargo build` (including `tauri dev`'s underlying build), so
/// `resource_dir()` works in both dev and a packaged app — but only after
/// at least one build has happened with the resource in place, and only
/// as of whatever `packs/` looked like at that build (editing files under
/// `packs/` needs a rebuild to show up, same as any other compiled-in
/// asset). The `CARGO_MANIFEST_DIR`-relative fallback exists for the
/// narrow gap before that first copy has happened.
fn builtin_packs_dir<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidate = resource_dir.join("packs");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }

    let dev_candidate = Path::new(env!("CARGO_MANIFEST_DIR")).join("../packs");
    if dev_candidate.is_dir() {
        return Some(dev_candidate);
    }

    None
}

fn user_packs_dir<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|dir| dir.join("packs"))
}

fn load_pack_dir(dir: &Path) -> Option<PackInfo> {
    let manifest_path = dir.join("pack.json");
    let contents = fs::read_to_string(&manifest_path).ok()?;
    let manifest: PackManifest = serde_json::from_str(&contents).ok()?;
    if validate_manifest(&manifest).is_err() {
        return None;
    }
    Some(PackInfo {
        manifest,
        dir: dir.to_string_lossy().to_string(),
    })
}

/// Lists every valid pack found in the bundled `packs/` directory and the
/// user's imported-packs directory. Invalid packs (bad/missing pack.json)
/// are silently skipped rather than failing the whole listing.
#[tauri::command]
pub fn list_packs<R: Runtime>(app: AppHandle<R>) -> Vec<PackInfo> {
    let mut packs = Vec::new();

    for base in [builtin_packs_dir(&app), user_packs_dir(&app)]
        .into_iter()
        .flatten()
    {
        // The frames renderer fetches frame images via the asset:// protocol,
        // which refuses anything outside its allow-listed scope. Packs can
        // live in the bundled resources dir or the user's app-data dir,
        // neither of which is known at build time (the user one especially,
        // since it's per-OS/per-user), so the scope is extended here at
        // runtime instead of being fully static in tauri.conf.json.
        let _ = app.asset_protocol_scope().allow_directory(&base, true);

        let Ok(entries) = fs::read_dir(&base) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(pack) = load_pack_dir(&path) {
                    packs.push(pack);
                }
            }
        }
    }

    packs
}

/// Opens a native file picker for a `.zip`, extracts it into the user
/// packs directory, and validates the result. Returns an error message on
/// failure (bad zip, missing/invalid pack.json) rather than leaving a
/// half-imported pack around — the extracted folder is removed on any
/// validation failure.
#[tauri::command]
pub async fn import_pack<R: Runtime>(app: AppHandle<R>) -> Result<Option<PackInfo>, String> {
    let file_path = {
        let (tx, rx) = tokio::sync::oneshot::channel();
        app.dialog()
            .file()
            .add_filter("Character pack", &["zip"])
            .pick_file(move |path| {
                let _ = tx.send(path);
            });
        rx.await.map_err(|_| "file picker closed unexpectedly".to_string())?
    };

    let Some(file_path) = file_path else {
        return Ok(None); // user cancelled
    };
    let zip_path = file_path
        .into_path()
        .map_err(|e| format!("invalid file path: {e}"))?;

    let packs_dir = user_packs_dir(&app).ok_or("could not resolve app data dir")?;
    fs::create_dir_all(&packs_dir).map_err(|e| format!("failed to create packs dir: {e}"))?;

    // Extract into a temp staging folder first, so a failed/invalid
    // import never corrupts an existing pack with the same id.
    let staging_dir = packs_dir.join(format!(".staging-{}", std::process::id()));
    if staging_dir.exists() {
        let _ = fs::remove_dir_all(&staging_dir);
    }
    fs::create_dir_all(&staging_dir).map_err(|e| format!("failed to create staging dir: {e}"))?;

    let extract_result = (|| -> Result<(), String> {
        let file = fs::File::open(&zip_path).map_err(|e| format!("failed to open zip: {e}"))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("not a valid zip file: {e}"))?;
        archive
            .extract(&staging_dir)
            .map_err(|e| format!("failed to extract zip: {e}"))
    })();

    if let Err(e) = extract_result {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err(e);
    }

    // The zip may or may not wrap its contents in a single top-level
    // folder; find wherever pack.json actually ended up.
    let manifest_dir = find_pack_json_dir(&staging_dir)
        .ok_or_else(|| "zip does not contain a pack.json".to_string())?;

    let pack = load_pack_dir(&manifest_dir).ok_or_else(|| {
        "pack.json is missing required fields or failed validation".to_string()
    })?;

    let final_dir = packs_dir.join(&pack.manifest.id);
    if final_dir.exists() {
        fs::remove_dir_all(&final_dir).map_err(|e| format!("failed to replace existing pack: {e}"))?;
    }
    fs::rename(&manifest_dir, &final_dir).map_err(|e| format!("failed to install pack: {e}"))?;
    let _ = fs::remove_dir_all(&staging_dir); // clean up any staging leftovers (e.g. wrapper folder)

    let mut pack = pack;
    pack.dir = final_dir.to_string_lossy().to_string();
    Ok(Some(pack))
}

fn find_pack_json_dir(root: &Path) -> Option<PathBuf> {
    if root.join("pack.json").exists() {
        return Some(root.to_path_buf());
    }
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_pack_json_dir(&path) {
                return Some(found);
            }
        }
    }
    None
}
