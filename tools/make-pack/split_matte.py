#!/usr/bin/env python3
"""Rebuilds a clip that carries its own alpha matte into a transparent clip.

A lot of stock character animation ships as one file holding the colour plate
first and a black-and-white alpha matte of the same animation second (the
"luma matte" convention). Colour-keying such a clip fails twice over: the key
can't cleanly cut a subject that shares tones with the plate, and the matte
half ends up treated as more footage.

Using the supplied matte instead gives a pixel-exact cutout with proper soft
edges — strictly better than any key we could tune by hand.

    ./tools/.venv/bin/python tools/make-pack/split_matte.py \\
      --src clips/batch2/118906.mp4 --out clips/alpha/droplet.mov

Writes a QuickTime file with a real alpha channel, ready to pass to
segment_clip.py and then make_pack.py with --bg none.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import tempfile
from pathlib import Path

import imageio_ffmpeg
import numpy as np
from PIL import Image

FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()


def probe_duration(path: Path) -> float:
    out = subprocess.run(
        [FFMPEG, "-hide_banner", "-i", str(path)], capture_output=True, text=True
    ).stderr
    for line in out.splitlines():
        if "Duration:" in line:
            h, m, s = line.split("Duration:")[1].split(",")[0].strip().split(":")
            return int(h) * 3600 + int(m) * 60 + float(s)
    raise RuntimeError(f"could not read duration of {path}")


def find_split(path: Path, duration: float, fps: int = 5) -> float | None:
    """Returns the timestamp where the colour plate gives way to the matte.

    The matte half has a solid black surround, so sampling a corner across the
    clip shows one sharp brightness drop at the boundary.
    """
    tmp = Path(tempfile.mkdtemp())
    try:
        subprocess.run(
            [FFMPEG, "-y", "-v", "error", "-i", str(path),
             "-vf", f"fps={fps},scale=160:-1", f"{tmp}/%05d.png"],
            check=True,
        )
        corners = []
        for f in sorted(tmp.iterdir()):
            arr = np.array(Image.open(f).convert("RGB"))
            corners.append(float(arr[:6, :6].mean()))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    if len(corners) < 4:
        return None

    # Look for the single biggest drop, in the middle 60% of the clip.
    lo, hi = int(len(corners) * 0.2), int(len(corners) * 0.8)
    best_i, best_drop = None, 0.0
    for i in range(max(1, lo), min(len(corners), hi)):
        drop = corners[i - 1] - corners[i]
        if drop > best_drop:
            best_drop, best_i = drop, i
    if best_i is None or best_drop < 60:
        return None

    split = best_i / fps
    # A true matte pairing splits the clip in half; anything far off that is
    # more likely a scene change, and merging it would misalign every frame.
    if abs(split - duration / 2) > duration * 0.08:
        return None
    return duration / 2


def build_alpha(src: Path, dest: Path, split: float, fps: int) -> int:
    tmp = Path(tempfile.mkdtemp())
    try:
        colour, matte = tmp / "c", tmp / "m"
        colour.mkdir(); matte.mkdir()
        subprocess.run(
            [FFMPEG, "-y", "-v", "error", "-t", f"{split:.3f}", "-i", str(src),
             "-vf", f"fps={fps}", f"{colour}/%05d.png"], check=True)
        subprocess.run(
            [FFMPEG, "-y", "-v", "error", "-ss", f"{split:.3f}", "-i", str(src),
             "-vf", f"fps={fps}", f"{matte}/%05d.png"], check=True)

        cs = sorted(colour.iterdir())
        ms = sorted(matte.iterdir())
        n = min(len(cs), len(ms))
        if n == 0:
            raise RuntimeError("no frames extracted")

        merged = tmp / "out"
        merged.mkdir()
        for i in range(n):
            col = Image.open(cs[i]).convert("RGB")
            mat = Image.open(ms[i]).convert("L").resize(col.size)
            col.putalpha(mat)
            col.save(merged / f"{i + 1:05d}.png")

        # qtrle keeps the alpha channel intact; most lossy codecs drop it.
        subprocess.run(
            [FFMPEG, "-y", "-v", "error", "-framerate", str(fps),
             "-i", f"{merged}/%05d.png", "-c:v", "qtrle", "-pix_fmt", "argb",
             str(dest)], check=True)
        return n
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--src", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--fps", type=int, default=24)
    parser.add_argument("--split", type=float, default=None,
                        help="Override the detected colour/matte boundary, in seconds.")
    args = parser.parse_args()

    if not args.src.exists():
        parser.error(f"no such clip: {args.src}")

    duration = probe_duration(args.src)
    split = args.split if args.split is not None else find_split(args.src, duration)
    if split is None:
        print(f"{args.src.name}: no alpha matte detected — key this one by colour instead")
        return 2

    args.out.parent.mkdir(parents=True, exist_ok=True)
    n = build_alpha(args.src, args.out, split, args.fps)
    print(f"{args.src.name}: matte at {split:.2f}s -> {n} RGBA frames -> {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
