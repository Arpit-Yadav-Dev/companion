#!/usr/bin/env python3
"""Builds character packs from a directory of stock clips, end to end.

Ties together the three steps a stock clip needs — alpha matte extraction
(split_matte.py) or chroma keying, slicing one loop into per-state takes
(segment_clip.py), and packing (make_pack.py) — and picks the key settings
per clip by sampling its own background rather than using one global guess.

    ./tools/.venv/bin/python tools/make-pack/batch_build.py \\
      --clips clips/batch2 --spec tools/make-pack/batch_spec.json

The spec names each clip and says what to call the character; anything not
listed is skipped, so curating the roster is an edit to the spec, not a
code change.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import imageio_ffmpeg
import numpy as np
from PIL import Image

FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
VENV_PY = ROOT / "tools" / ".venv" / "bin" / "python"


def sample_background(clip: Path) -> tuple[tuple[int, int, int], float, float]:
    """Returns (rgb, spread, coverage) for the clip's background.

    Sampled from the frame corners, which is where the plate shows even when
    the subject fills the middle. `spread` near zero means a flat colour that
    will key cleanly; a large value means a gradient or a real scene.
    """
    tmp = Path(tempfile.mkdtemp())
    try:
        subprocess.run(
            [FFMPEG, "-y", "-v", "error", "-i", str(clip),
             "-vf", r"select=eq(n\,20),scale=320:-1", "-vframes", "1",
             str(tmp / "a.png")], check=True)
        arr = np.array(Image.open(tmp / "a.png").convert("RGB"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    corners = np.vstack([
        arr[:8, :8].reshape(-1, 3), arr[:8, -8:].reshape(-1, 3),
        arr[-8:, :8].reshape(-1, 3), arr[-8:, -8:].reshape(-1, 3),
    ])
    bg = corners.mean(axis=0)
    spread = float(corners.std(axis=0).mean())
    dist = np.abs(arr.astype(int) - bg.astype(int)).sum(axis=2)
    coverage = float((dist < 40).mean())
    return tuple(int(c) for c in bg), spread, coverage


def key_settings(rgb: tuple[int, int, int]) -> tuple[str, str, float, float]:
    """Picks colorkey hex, despill channel and tolerances for a plate colour.

    Tolerance is the whole game here. Too loose and it eats into a subject
    that shares tones with the plate (a navy suit on black, grey fur on
    teal); too tight and a halo of plate colour survives at the edges. Dark
    and desaturated plates sit closest to real subject colours, so they get
    the tightest tolerance.
    """
    r, g, b = rgb
    hex_code = f"0x{r:02X}{g:02X}{b:02X}"
    brightness = (r + g + b) / 3
    saturation = max(rgb) - min(rgb)

    # Despill removes the fringe a bright plate casts onto edges. It is only
    # worth doing for a strongly saturated screen, and picking the wrong
    # channel tints the subject (blue despill turned a dancer's jeans purple).
    if saturation > 60 and b > r and b > g:
        despill = "blue"
    else:
        despill = "green"

    if saturation > 90:          # a real green/blue screen: safe to key wide
        return hex_code, despill, 0.16, 0.05
    if brightness > 200:         # white plate: subjects rarely go pure white
        return hex_code, "green", 0.10, 0.04
    # dark or muted plate: closest to subject tones, so stay tight
    return hex_code, "green", 0.06, 0.02


def run(cmd: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True)


def build_one(clip: Path, slug: str, name: str, work: Path,
              height: int, fps: int, quality: int) -> tuple[bool, str]:
    seg_dir = work / "segments" / slug

    # An embedded alpha matte beats any key we could tune, so always try it.
    alpha_clip = work / "alpha" / f"{slug}.mov"
    alpha_clip.parent.mkdir(parents=True, exist_ok=True)
    matte = run([str(VENV_PY), str(HERE / "split_matte.py"),
                 "--src", str(clip), "--out", str(alpha_clip), "--fps", str(fps)])

    if matte.returncode == 0:
        source, bg_args = alpha_clip, ["--bg", "none"]
        how = "alpha matte"
    else:
        rgb, spread, coverage = sample_background(clip)
        if spread > 3.0 or coverage < 0.55:
            return False, f"background is not a flat plate (spread {spread:.1f}, cover {coverage*100:.0f}%)"
        hex_code, despill, sim, blend = key_settings(rgb)
        source = clip
        bg_args = ["--bg", "colorkey", "--colorkey-hex", hex_code,
                   "--despill", despill,
                   "--colorkey-similarity", str(sim),
                   "--colorkey-blend", str(blend)]
        how = f"key {hex_code} sim={sim}"

    seg = run([str(VENV_PY), str(HERE / "segment_clip.py"),
               "--src", str(source), "--out", str(seg_dir)])
    if seg.returncode != 0:
        return False, f"segment failed: {seg.stderr.strip()[:120]}"

    ext = ".mov" if source.suffix.lower() == ".mov" else ".mp4"
    clip_args = []
    for state in ["walk", "idle", "ask", "drink", "shrug", "sad", "wave"]:
        clip_args += ["--clip", f"{state}={seg_dir / (state + ext)}"]

    shutil.rmtree(ROOT / "packs" / slug, ignore_errors=True)
    pack = run([str(VENV_PY), str(HERE / "make_pack.py"),
                "--name", name, "--id", slug, *clip_args,
                "--height", str(height), "--fps", str(fps),
                "--loop-seconds", "2.5", "--quality", str(quality), *bg_args])
    if pack.returncode != 0:
        return False, f"pack failed: {pack.stderr.strip()[:160]}"

    return True, how


def verify(slug: str) -> tuple[bool, str]:
    """Rejects packs the key visibly damaged.

    A pack can build fine and still be unusable: if the key ate the subject
    the frames come out mostly see-through, and if it removed nothing the
    subject fills the entire frame. Both are obvious in the alpha channel,
    so check here rather than leaving it for the eye to catch later.
    """
    pack_dir = Path(ROOT / "packs" / slug)
    frames = sorted((pack_dir / "idle").glob("*.webp"))
    if not frames:
        return False, "no frames written"

    opaque_ratios, edge_hits = [], 0
    sample = frames[:: max(1, len(frames) // 6)][:6]
    for f in sample:
        alpha = np.array(Image.open(f).convert("RGBA").split()[-1])
        solid = alpha > 128
        opaque_ratios.append(solid.mean())
        h, w = solid.shape
        border = np.concatenate([solid[0], solid[-1], solid[:, 0], solid[:, -1]])
        if border.mean() > 0.25:
            edge_hits += 1

    mean_opaque = float(np.mean(opaque_ratios))
    if mean_opaque < 0.04:
        return False, f"subject keyed away (only {mean_opaque*100:.1f}% opaque)"
    if edge_hits >= len(sample) - 1:
        return False, "background not removed (subject touches every edge)"
    return True, f"{mean_opaque*100:.0f}% opaque"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--clips", required=True, type=Path)
    parser.add_argument("--spec", required=True, type=Path)
    parser.add_argument("--height", type=int, default=360)
    parser.add_argument("--fps", type=int, default=20)
    parser.add_argument("--quality", type=int, default=72)
    parser.add_argument("--only", default=None, help="Comma-separated slugs to rebuild")
    args = parser.parse_args()

    spec = json.loads(args.spec.read_text())
    only = {s.strip() for s in args.only.split(",")} if args.only else None
    work = ROOT / "clips"

    built, failed = [], []
    for entry in spec:
        slug, name, vid = entry["id"], entry["name"], str(entry["clip"])
        if only and slug not in only:
            continue
        clip = args.clips / f"{vid}.mp4"
        if not clip.exists():
            failed.append((slug, "clip missing")); continue

        ok, detail = build_one(clip, slug, name, work,
                               args.height, args.fps, args.quality)
        if ok:
            ok, detail2 = verify(slug)
            detail = f"{detail} -> {detail2}"
            if not ok:
                shutil.rmtree(ROOT / "packs" / slug, ignore_errors=True)

        if ok:
            size = sum(p.stat().st_size for p in (ROOT / "packs" / slug).rglob("*")) / 1e6
            built.append((slug, f"{size:4.1f}MB  {detail}"))
            print(f"  OK   {slug:<12} {size:5.1f}MB  {detail}", flush=True)
        else:
            failed.append((slug, detail))
            print(f"  SKIP {slug:<12} {detail}", flush=True)

    print(f"\nbuilt {len(built)}, skipped {len(failed)}")
    for slug, why in failed:
        print(f"  - {slug}: {why}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
