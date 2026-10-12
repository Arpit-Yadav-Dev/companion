#!/usr/bin/env python3
"""Turns AI-generated video clips into a Companion character pack.

Run via the project-local venv, never the system Python:

    ./tools/.venv/bin/python tools/make-pack/make_pack.py \\
      --name "Aakash (3D)" --id aakash-3d \\
      --clip walk=clips/walk.mp4 --clip idle=clips/idle.mp4 \\
      --clip ask=clips/ask.mp4 --clip drink=clips/drink.mp4 \\
      --clip shrug=clips/shrug.mp4 --clip sad=clips/sad.mp4 \\
      --clip wave=clips/wave.mp4 \\
      --height 420 --fps 24 --bg auto

See README.md in this directory for how to generate the clips themselves.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
import zipfile
from dataclasses import dataclass
from pathlib import Path

import imageio_ffmpeg
import numpy as np
from PIL import Image

FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()

# Required states -> (default clip name, loops?). The clip name is what
# you pass as --clip <name>=path.mp4; it also becomes the frame directory
# name inside the pack (so pack.json's states.happy.dir == "drink", not
# "happy" — matching the Sip/Pixel built-in packs' convention of naming
# directories after the physical action, not the abstract state).
STATE_DEFAULT_CLIP = {
    "walk_in": ("walk", False),
    "idle": ("idle", True),
    "ask": ("ask", True),
    "happy": ("drink", False),
    "shrug": ("shrug", False),
    "sad": ("sad", False),
    "wave": ("wave", False),
    "walk_out": ("walk", False),
}
REQUIRED_STATES = list(STATE_DEFAULT_CLIP.keys())

MAX_PACK_SIZE_MB = 15


@dataclass
class ClipFrames:
    state: str
    frame_paths: list[Path]  # extracted, (optionally) bg-removed PNGs


def run_ffmpeg(args: list[str]) -> None:
    result = subprocess.run(
        [FFMPEG, "-y", "-hide_banner", "-loglevel", "error", *args],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"ffmpeg failed: {result.stderr.strip()}")


def extract_frames(
    clip_path: Path,
    out_dir: Path,
    fps: int,
    bg_mode: str,
    colorkey_hex: str,
    despill_type: str,
    colorkey_similarity: float = 0.20,
    colorkey_blend: float = 0.08,
) -> list[Path]:
    """Extracts frames at the target fps, applying chroma-key removal
    during extraction when bg_mode is "auto" or "colorkey". Frames land as
    sequential zero-padded PNGs (lossless, since later steps still need to
    crop/align before the final lossy WebP export)."""
    out_dir.mkdir(parents=True, exist_ok=True)
    pattern = str(out_dir / "%04d.png")

    vf_parts = [f"fps={fps}"]
    if bg_mode in ("auto", "colorkey"):
        # similarity/blend default tight enough that light-coloured
        # subjects (e.g. a pale dog on a pale-blue screen) don't get
        # partially keyed out themselves — a wide net (e.g. 0.35) picks up
        # the subject's own near-background tones and leaves them
        # semi-transparent instead of solid. Widen via --colorkey-similarity
        # only if the actual background doesn't key out cleanly.
        vf_parts.append(f"colorkey={colorkey_hex}:{colorkey_similarity}:{colorkey_blend}")
        vf_parts.append(f"despill=type={despill_type}")
    run_ffmpeg([
        "-i", str(clip_path),
        "-vf", ",".join(vf_parts),
        "-pix_fmt", "rgba",
        pattern,
    ])

    frames = sorted(out_dir.glob("*.png"))
    if not frames:
        raise RuntimeError(f"no frames extracted from {clip_path}")
    return frames


def remove_background_rembg(frame_paths: list[Path]) -> None:
    """Runs each frame through rembg in place. Imported lazily so the
    default (chroma-key) path never requires the heavy optional
    dependency to be installed."""
    try:
        from rembg import remove  # type: ignore[import-not-found]
    except ImportError as e:
        raise RuntimeError(
            "rembg is not installed. Run:\n"
            "  ./tools/.venv/bin/pip install -r tools/make-pack/requirements-rembg.txt"
        ) from e

    for path in frame_paths:
        data = path.read_bytes()
        result = remove(data)
        path.write_bytes(result)


def non_transparent_bbox(image: Image.Image) -> tuple[int, int, int, int] | None:
    """Returns (left, top, right, bottom) of the non-transparent pixel
    extent, or None if the frame is fully transparent."""
    alpha = np.array(image.split()[-1])
    # A chroma key almost always leaves a scatter of faint pixels across the
    # plate. Testing "any pixel above 10" lets a handful of those stray pixels
    # pin the box to the full frame, so the character ends up tiny inside a
    # mostly-empty crop. Require a row/column to carry a real slice of the
    # subject instead: solidly opaque pixels, and more than a couple of them.
    solid = alpha > 128
    min_run = max(2, int(0.004 * max(alpha.shape)))
    rows = solid.sum(axis=1) >= min_run
    cols = solid.sum(axis=0) >= min_run
    if not rows.any() or not cols.any():
        return None
    top, bottom = np.where(rows)[0][[0, -1]]
    left, right = np.where(cols)[0][[0, -1]]
    return int(left), int(top), int(right) + 1, int(bottom) + 1


def compute_shared_bbox(all_frame_paths: list[Path]) -> tuple[int, int, int, int]:
    """A single bounding box covering every frame across every state, so
    the character doesn't jitter between states that happen to have
    slightly different crops. Also used to align feet to the bottom."""
    union_left = union_top = None
    union_right = union_bottom = None

    for path in all_frame_paths:
        with Image.open(path) as img:
            img = img.convert("RGBA")
            bbox = non_transparent_bbox(img)
        if bbox is None:
            continue
        left, top, right, bottom = bbox
        union_left = left if union_left is None else min(union_left, left)
        union_top = top if union_top is None else min(union_top, top)
        union_right = right if union_right is None else max(union_right, right)
        union_bottom = bottom if union_bottom is None else max(union_bottom, bottom)

    if union_left is None:
        raise RuntimeError("every extracted frame was fully transparent — check --bg mode")

    return union_left, union_top, union_right, union_bottom


def crop_resize_export_webp(
    frame_paths: list[Path],
    bbox: tuple[int, int, int, int],
    target_height: int,
    out_dir: Path,
    quality: int = 80,
) -> list[Path]:
    out_dir.mkdir(parents=True, exist_ok=True)
    left, top, right, bottom = bbox
    crop_width = right - left
    crop_height = bottom - top
    scale = target_height / crop_height
    target_width = max(1, round(crop_width * scale))

    out_paths = []
    for i, path in enumerate(frame_paths):
        with Image.open(path) as img:
            img = img.convert("RGBA").crop(bbox)
            img = img.resize((target_width, target_height), Image.LANCZOS)
            out_path = out_dir / f"{i + 1:04d}.webp"
            img.save(out_path, "WEBP", quality=quality, lossless=False)
            out_paths.append(out_path)
    return out_paths


def detect_loop_seam(frame_paths: list[Path]) -> int:
    """For a looping state, finds which frame (other than the first) is
    the closest visual match to frame 0, so the pack can trim to that
    frame for a seamless loop. Returns the index of the best loop-end
    frame; the caller decides whether to actually trim (kept simple here:
    callers currently just log this, since aggressive trimming risks
    cutting a clip too short for an AI-generated source that doesn't
    loop perfectly)."""
    if len(frame_paths) < 3:
        return len(frame_paths) - 1

    with Image.open(frame_paths[0]) as first:
        first_arr = np.array(first.convert("RGB").resize((64, 64)), dtype=np.int16)

    best_index = len(frame_paths) - 1
    best_diff = float("inf")
    # Only search the back half of the clip — the loop point should be
    # near the end, not immediately after frame 0.
    search_start = len(frame_paths) // 2
    for i in range(search_start, len(frame_paths)):
        with Image.open(frame_paths[i]) as frame:
            arr = np.array(frame.convert("RGB").resize((64, 64)), dtype=np.int16)
        diff = float(np.mean(np.abs(arr - first_arr)))
        if diff < best_diff:
            best_diff = diff
            best_index = i
    return best_index


def build_pack_json(
    pack_id: str,
    name: str,
    fps: int,
    height: int,
    states: dict[str, str],
    frame_counts: dict[str, int],
    frame_width: int,
) -> dict:
    return {
        "id": pack_id,
        "name": name,
        "renderer": "frames",
        "fps": fps,
        "heightPx": height,
        "widthPx": frame_width,
        "anchor": {"x": 0.5, "y": 1.0},
        "bubbleOffset": {"x": 0, "y": -0.95},
        "states": {
            state_name: {
                "dir": dir_name,
                "loop": STATE_DEFAULT_CLIP[state_name][1],
                "frameCount": frame_counts[dir_name],
                **(
                    {"move": "enter"}
                    if state_name == "walk_in"
                    else {"move": "exit"}
                    if state_name == "walk_out"
                    else {}
                ),
            }
            for state_name, dir_name in states.items()
        },
        "props": {},
        "mirrorForDirection": True,
    }


def zip_pack(pack_dir: Path, out_zip: Path) -> None:
    with zipfile.ZipFile(out_zip, "w", zipfile.ZIP_DEFLATED) as zf:
        for file_path in sorted(pack_dir.rglob("*")):
            if file_path.is_file():
                zf.write(file_path, file_path.relative_to(pack_dir.parent))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--name", required=True, help='Display name, e.g. "Aakash (3D)"')
    parser.add_argument("--id", required=True, help="Pack id, lowercase-with-dashes")
    parser.add_argument(
        "--clip",
        action="append",
        required=True,
        metavar="name=path",
        help="Repeatable. e.g. --clip walk=clips/walk.mp4. Clip names and "
        "the state each feeds: walk -> walk_in & walk_out, idle -> idle, "
        "ask -> ask, drink -> happy, shrug -> shrug, sad -> sad, "
        "wave -> wave. All seven are required.",
    )
    parser.add_argument("--height", type=int, default=420, help="Target height in px (default 420)")
    parser.add_argument("--fps", type=int, default=24, help="Target fps (default 24)")
    parser.add_argument(
        "--loop-seconds",
        type=float,
        default=4.0,
        help="How many seconds of the source clip to keep for looping states "
        "(idle/ask) (default 4). These dominate pack size, and a few seconds "
        "reads the same once looping.",
    )
    parser.add_argument(
        "--quality",
        type=int,
        default=80,
        help="WebP quality 1-100 (default 80). Lower trades visible fidelity "
        "for a smaller download.",
    )
    parser.add_argument(
        "--bg",
        choices=["auto", "colorkey", "rembg", "none"],
        default="auto",
        help="auto/colorkey = solid-colour-screen chroma key (default, "
        "green unless --colorkey-hex/--despill say otherwise); "
        "rembg = ML background removal for non-solid-colour clips; "
        "none = clips are already transparent/pre-keyed",
    )
    parser.add_argument(
        "--colorkey-hex",
        default="0x00FF00",
        help="Background colour to key out, as 0xRRGGBB (default: green, 0x00FF00). "
        "Sample a background pixel from your clip to find the right value.",
    )
    parser.add_argument(
        "--despill",
        choices=["green", "blue"],
        default="green",
        help="Which colour fringe to remove from edges after keying (default: green). "
        "Set to blue when --colorkey-hex is a blue screen.",
    )
    parser.add_argument(
        "--colorkey-similarity",
        type=float,
        default=0.20,
        help="How close a pixel must be to --colorkey-hex to be keyed out (0-1, default "
        "0.20). Lower this if a light-coloured subject is coming out semi-transparent; "
        "raise it if background colour is still visible at the edges.",
    )
    parser.add_argument(
        "--colorkey-blend",
        type=float,
        default=0.08,
        help="Softness of the chroma-key edge falloff (0-1, default 0.08).",
    )
    parser.add_argument("--out", default=None, help="Output directory (default: ./packs/<id>)")
    args = parser.parse_args()

    clip_paths: dict[str, Path] = {}
    for item in args.clip:
        if "=" not in item:
            parser.error(f"--clip must be name=path, got: {item}")
        clip_name, path_str = item.split("=", 1)
        clip_paths[clip_name] = Path(path_str)

    required_clip_names = {clip_name for clip_name, _ in STATE_DEFAULT_CLIP.values()}
    missing = sorted(required_clip_names - clip_paths.keys())
    if missing:
        print(f"error: missing required clips: {', '.join(missing)}", file=sys.stderr)
        return 1

    for clip_name, path in clip_paths.items():
        if not path.exists():
            print(f"error: clip '{clip_name}' not found: {path}", file=sys.stderr)
            return 1

    out_root = Path(args.out) if args.out else Path("packs") / args.id
    if out_root.exists():
        print(f"error: {out_root} already exists — remove it first or pass --out", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="make-pack-") as tmp:
        tmp_path = Path(tmp)
        print(f"Extracting frames at {args.fps} fps...")

        # Extract each unique clip once (the "walk" clip feeds two states
        # but should only be decoded/keyed a single time).
        clips_by_name: dict[str, ClipFrames] = {}
        for clip_name, clip_path in clip_paths.items():
            raw_dir = tmp_path / "raw" / clip_name
            frames = extract_frames(
                clip_path,
                raw_dir,
                args.fps,
                args.bg,
                args.colorkey_hex,
                args.despill,
                args.colorkey_similarity,
                args.colorkey_blend,
            )
            if args.bg == "rembg":
                remove_background_rembg(frames)
            clips_by_name[clip_name] = ClipFrames(state=clip_name, frame_paths=frames)
            print(f"  {clip_name}: {len(frames)} frames")

        print("Computing shared crop box across all states...")
        all_frames = [p for c in clips_by_name.values() for p in c.frame_paths]
        bbox = compute_shared_bbox(all_frames)
        print(f"  shared bbox: {bbox}")

        print(f"Cropping, resizing to {args.height}px, exporting WebP...")
        state_dirs: dict[str, str] = {}
        exported_dirs: set[str] = set()
        frame_counts: dict[str, int] = {}
        frame_width = max(1, round((bbox[2] - bbox[0]) * (args.height / (bbox[3] - bbox[1]))))
        # A walk-in/out should finish in about as long as the director's
        # fixed WALK_MS entrance animation (~1.2s) — reusing a long clip
        # (e.g. 12s of dancing) for "walk" means the CSS slide-in finishes
        # long before the frames do, so the character seems to pop into
        # place instead of walking in. Cap walk-family clips; other states
        # (idle/ask/etc.) can stay long since they loop or play once at
        # their own pace.
        MAX_WALK_FRAMES = max(1, round(1.2 * args.fps))
        # One-shot reaction states (happy/shrug/sad/wave) play once and
        # move on — capping them to ~2s keeps a reused/long source clip
        # from making every reaction feel like a long pause. Looping
        # states (idle/ask) are untouched since any length loops fine.
        MAX_ONESHOT_FRAMES = max(1, round(2.0 * args.fps))
        ONESHOT_STATES = {"happy", "shrug", "sad", "wave"}
        # Looping states dominate pack size — a 12s source clip gives ~300
        # frames for idle and another ~300 for ask, which was over 80% of
        # the total. A few seconds reads the same once it's looping, and
        # these packs are downloaded over the network, so the saving
        # matters more than the extra variation.
        MAX_LOOP_FRAMES = max(1, round(args.loop_seconds * args.fps))
        for state_name, (clip_name, loops) in STATE_DEFAULT_CLIP.items():
            state_dirs[state_name] = clip_name
            if clip_name in exported_dirs:
                continue  # already exported (e.g. "walk" feeds two states)
            exported_dirs.add(clip_name)

            source_frames = clips_by_name[clip_name].frame_paths
            if clip_name == "walk" and len(source_frames) > MAX_WALK_FRAMES:
                source_frames = source_frames[:MAX_WALK_FRAMES]
            elif state_name in ONESHOT_STATES and len(source_frames) > MAX_ONESHOT_FRAMES:
                source_frames = source_frames[:MAX_ONESHOT_FRAMES]
            elif loops and len(source_frames) > MAX_LOOP_FRAMES:
                source_frames = source_frames[:MAX_LOOP_FRAMES]

            out_dir = tmp_path / "pack" / clip_name
            webp_frames = crop_resize_export_webp(
                source_frames, bbox, args.height, out_dir, args.quality
            )
            frame_counts[clip_name] = len(webp_frames)

            if loops:
                seam = detect_loop_seam(webp_frames)
                print(f"  {clip_name}: loop seam near frame {seam + 1}/{len(webp_frames)}")

        manifest = build_pack_json(
            args.id, args.name, args.fps, args.height, state_dirs, frame_counts, frame_width
        )
        pack_dir = tmp_path / "pack"
        (pack_dir / "pack.json").write_text(json.dumps(manifest, indent=2))

        out_root.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(pack_dir, out_root)

    # Write the shareable .zip outside the pack directory. Anything left
    # inside packs/ gets bundled into the app by tauri.conf.json's
    # resources entry, and the app only ever reads zips the user picks
    # via the import dialog — a zip sitting next to the pack folder is
    # pure dead weight that silently doubles each frames pack's cost.
    # Anchor on the repo root rather than walking up from --out, so a
    # custom output path doesn't nest the zip under itself.
    zip_dir = Path(__file__).resolve().parents[2] / "dist" / "packs"
    zip_dir.mkdir(parents=True, exist_ok=True)
    # Name by pack id, not the output folder — the id is what the app and
    # the download catalog key on.
    zip_path = zip_dir / f"{args.id}.zip"
    zip_pack(out_root, zip_path)

    total_bytes = sum(f.stat().st_size for f in out_root.rglob("*") if f.is_file())
    total_mb = total_bytes / (1024 * 1024)
    print(f"\nPack written to {out_root}/ ({total_mb:.1f} MB)")
    print(f"Zipped to {zip_path} ({zip_path.stat().st_size / (1024 * 1024):.1f} MB)")
    if total_mb > MAX_PACK_SIZE_MB:
        print(
            f"warning: pack is {total_mb:.1f} MB, over the {MAX_PACK_SIZE_MB} MB guideline — "
            "consider shorter clips, lower fps, or a smaller --height"
        )

    return 0


if __name__ == "__main__":
    sys.exit(main())
