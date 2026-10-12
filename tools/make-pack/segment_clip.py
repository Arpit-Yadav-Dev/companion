#!/usr/bin/env python3
"""Cuts one single-action stock clip into the per-state clips make_pack wants.

Most free stock character animations are a single looping action — a dance, a
walk cycle, an idle sway — rather than the seven separate takes make_pack.py
expects. This splits such a clip into those takes by slicing different time
ranges and varying playback speed, so the states still read as distinct
performances instead of the same loop eight times.

    ./tools/.venv/bin/python tools/make-pack/segment_clip.py \\
      --src clips/batch2/118906.mp4 --out clips/segments/drop

Then pass the results to make_pack.py as --clip walk=.../walk.mp4 etc.
"""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

import imageio_ffmpeg

FFMPEG = imageio_ffmpeg.get_ffmpeg_exe()

# state clip -> (start fraction, duration seconds, speed multiplier)
# Fractions are of the source duration, so one table fits clips of any length.
# Speed is what mostly sells the difference: an excited 1.35x reads as "happy"
# and a 0.7x drag reads as "sad" even when the underlying motion is the same.
SEGMENTS: dict[str, tuple[float, float, float]] = {
    "walk":  (0.00, 1.60, 1.15),
    "idle":  (0.25, 3.00, 0.85),
    "ask":   (0.45, 2.60, 1.00),
    "drink": (0.10, 1.80, 1.35),
    "shrug": (0.60, 1.30, 1.00),
    "sad":   (0.70, 1.60, 0.70),
    "wave":  (0.35, 1.50, 1.20),
}


def probe_duration(path: Path) -> float:
    out = subprocess.run(
        [FFMPEG, "-hide_banner", "-i", str(path)],
        capture_output=True, text=True,
    ).stderr
    for line in out.splitlines():
        if "Duration:" in line:
            stamp = line.split("Duration:")[1].split(",")[0].strip()
            h, m, s = stamp.split(":")
            return int(h) * 3600 + int(m) * 60 + float(s)
    raise RuntimeError(f"could not read duration of {path}")


def cut(src: Path, dest: Path, start: float, dur: float, speed: float) -> None:
    # setpts rescales presentation timestamps: <1 speeds up, >1 slows down.
    run = [
        FFMPEG, "-y", "-hide_banner", "-loglevel", "error",
        "-ss", f"{start:.3f}", "-t", f"{dur:.3f}", "-i", str(src),
        "-filter:v", f"setpts={1 / speed:.4f}*PTS",
    ]
    # An RGBA source has to stay RGBA: the default mp4 encoder would drop the
    # alpha channel without warning, undoing split_matte.py's work.
    if dest.suffix.lower() == ".mov":
        run += ["-c:v", "qtrle", "-pix_fmt", "argb"]
    run += ["-an", str(dest)]
    result = subprocess.run(run, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"ffmpeg failed on {dest.name}: {result.stderr.strip()}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--src", required=True, type=Path, help="Source clip")
    parser.add_argument("--out", required=True, type=Path, help="Directory for the state clips")
    args = parser.parse_args()

    if not args.src.exists():
        parser.error(f"no such clip: {args.src}")

    total = probe_duration(args.src)
    args.out.mkdir(parents=True, exist_ok=True)

    for name, (frac, dur, speed) in SEGMENTS.items():
        start = frac * total
        # Keep the window inside the clip; short sources just reuse the tail.
        if start + dur > total:
            start = max(0.0, total - dur)
        clip_dur = min(dur, total - start)
        ext = args.src.suffix.lower() if args.src.suffix.lower() == ".mov" else ".mp4"
        cut(args.src, args.out / f"{name}{ext}", start, clip_dur, speed)
        print(f"  {name:<6} {start:5.2f}s +{clip_dur:4.2f}s @{speed:.2f}x")

    print(f"wrote 7 state clips to {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
