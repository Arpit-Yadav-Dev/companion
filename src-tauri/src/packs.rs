//! Character pack discovery, validation, and import. A pack is a folder
//! with `pack.json` + assets, living either in the bundled `packs/`
//! directory (shipped with the app) or the user packs directory under the
//! app data dir (imported via .zip, or downloaded from the catalog).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_dialog::DialogExt;

pub const REQUIRED_STATES: &[&str] = &[
    "walk_in", "idle", "ask", "happy", "shrug", "sad", "wave", "walk_out",
];

/// Catalog of characters available to download rather than ship in the
/// app. Video-based packs are several MB each, so bundling them would
/// bloat the installer for characters most users won't pick — they're
/// hosted as GitHub release assets and fetched on demand instead.
const CATALOG_BASE_URL: &str =
    "https://github.com/Arpit-Yadav-Dev/companion/releases/download/characters-v1";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Approximate download size, for showing before the user commits.
    pub size_mb: f32,
    pub url: String,
    /// Bundled still frame (a few KB) so the card can show what the
    /// character looks like before anything is downloaded.
    pub preview: String,
}

fn catalog() -> Vec<CatalogEntry> {
    [
        ("dancing-dog", "Dancing Dog", "A 3D cartoon dog with plenty of energy", 2.4),
        ("ellie", "Ellie", "A cheerful pink cartoon elephant", 2.6),
    ]
    .into_iter()
    .map(|(id, name, description, size_mb)| CatalogEntry {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        size_mb,
        url: format!("{CATALOG_BASE_URL}/{id}.zip"),
        preview: format!("/catalog/{id}.webp"),
    })
    .collect()
}

/// The downloadable characters, each flagged with whether it's already
/// installed so the UI can show "Download" vs nothing.
#[tauri::command]
pub fn list_catalog<R: Runtime>(app: AppHandle<R>) -> Vec<CatalogEntryStatus> {
    let installed: Vec<String> = list_packs(app)
        .into_iter()
        .map(|p| p.manifest.id)
        .collect();

    catalog()
        .into_iter()
        .map(|entry| CatalogEntryStatus {
            installed: installed.contains(&entry.id),
            entry,
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntryStatus {
    #[serde(flatten)]
    pub entry: CatalogEntry,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    id: String,
    /// 0-100, or None when the server didn't send a content-length and we
    /// can't know the total ahead of time.
    percent: Option<u8>,
}

/// Downloads a catalog character and installs it, emitting
/// `pack://download-progress` as it goes so the UI can show a bar.
#[tauri::command]
pub async fn download_pack<R: Runtime>(app: AppHandle<R>, id: String) -> Result<PackInfo, String> {
    let entry = catalog()
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("unknown character: {id}"))?;

    let response = reqwest::get(&entry.url)
        .await
        .map_err(|e| format!("could not reach the download server: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "download failed with status {} — the character may have been moved",
            response.status()
        ));
    }

    let total = response.content_length();

    // Stream to a temp file rather than buffering several MB in memory.
    let tmp_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("could not resolve cache dir: {e}"))?;
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("could not create cache dir: {e}"))?;
    let tmp_zip = tmp_dir.join(format!("{id}-download.zip"));

    let download_result = (|| async {
        let mut file =
            fs::File::create(&tmp_zip).map_err(|e| format!("could not write download: {e}"))?;
        let mut downloaded: u64 = 0;
        let mut last_percent: Option<u8> = None;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("download interrupted: {e}"))?;
            file.write_all(&chunk)
                .map_err(|e| format!("could not write download: {e}"))?;
            downloaded += chunk.len() as u64;

            let percent = total.map(|t| ((downloaded * 100) / t.max(1)).min(100) as u8);
            // Only emit on change, so a fast download doesn't flood the
            // frontend with hundreds of identical events.
            if percent != last_percent {
                last_percent = percent;
                let _ = app.emit(
                    "pack://download-progress",
                    DownloadProgress { id: id.clone(), percent },
                );
            }
        }
        Ok::<(), String>(())
    })()
    .await;

    let result = download_result.and_then(|()| install_pack_from_zip(&app, &tmp_zip));
    let _ = fs::remove_file(&tmp_zip);
    result
}

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
    /// Frame count, written by make_pack.py. Lets the frontend fetch all
    /// of a state's frames in parallel instead of sequentially probing
    /// URLs until one 404s.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_count: Option<u32>,
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
    /// Real frame width in px, written by make_pack.py for `frames` packs.
    /// `svg` packs don't set this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width_px: Option<u32>,
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

    install_pack_from_zip(&app, &zip_path).map(Some)
}

/// Extracts a `.zip` into the user packs directory and validates it.
/// Shared by the file-picker import and the download flow so both get the
/// same staging/validation/rollback behaviour.
fn install_pack_from_zip<R: Runtime>(app: &AppHandle<R>, zip_path: &Path) -> Result<PackInfo, String> {
    let packs_dir = user_packs_dir(app).ok_or("could not resolve app data dir")?;
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

    // Validation and install, with the staging directory torn down on
    // every exit path — an early `?` here used to leave it behind.
    let installed = (|| -> Result<PackInfo, String> {
        // The zip may or may not wrap its contents in a single top-level
        // folder; find wherever pack.json actually ended up.
        let manifest_dir = find_pack_json_dir(&staging_dir)
            .ok_or_else(|| "zip does not contain a pack.json".to_string())?;

        let pack = load_pack_dir(&manifest_dir).ok_or_else(|| {
            "pack.json is missing required fields or failed validation".to_string()
        })?;

        let final_dir = packs_dir.join(&pack.manifest.id);
        if final_dir.exists() {
            fs::remove_dir_all(&final_dir)
                .map_err(|e| format!("failed to replace existing pack: {e}"))?;
        }
        fs::rename(&manifest_dir, &final_dir)
            .map_err(|e| format!("failed to install pack: {e}"))?;

        let mut pack = pack;
        pack.dir = final_dir.to_string_lossy().to_string();
        Ok(pack)
    })();

    let _ = fs::remove_dir_all(&staging_dir);
    installed
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
