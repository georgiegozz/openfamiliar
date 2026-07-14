"""Validate the canonical 4x4 mascot sheet against pixel-art release invariants."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SHEET = ROOT / "mascots" / "perrito-tech" / "assets" / "perrito-tech-spritesheet.png"
SHEET_SIZE = 256
FRAME_SIZE = 64
FRAME_COUNT = 16


def validate_sheet(path: Path) -> dict[str, object]:
    image = Image.open(path)
    errors: list[str] = []
    if image.mode != "RGBA":
        errors.append(f"mode must be RGBA, got {image.mode}")
    image = image.convert("RGBA")
    if image.size != (SHEET_SIZE, SHEET_SIZE):
        errors.append(f"size must be 256x256, got {image.size!r}")
    if errors:
        raise ValueError("; ".join(errors))

    alpha_values = set(image.getchannel("A").getdata())
    if not alpha_values.issubset({0, 255}) or alpha_values != {0, 255}:
        errors.append("alpha must contain both transparent and opaque pixels and no partial values")

    frame_hashes: list[str] = []
    coverage: list[float] = []
    for index in range(FRAME_COUNT):
        left = (index % 4) * FRAME_SIZE
        top = (index // 4) * FRAME_SIZE
        frame = image.crop((left, top, left + FRAME_SIZE, top + FRAME_SIZE))
        alpha = frame.getchannel("A")
        opaque = sum(1 for value in alpha.getdata() if value == 255)
        ratio = opaque / (FRAME_SIZE * FRAME_SIZE)
        coverage.append(round(ratio, 4))
        if ratio < 0.02:
            errors.append(f"frame {index} is empty or nearly empty")
        if ratio > 0.80:
            errors.append(f"frame {index} exceeds the maximum content coverage")
        border = []
        border.extend(alpha.crop((0, 0, FRAME_SIZE, 1)).getdata())
        border.extend(alpha.crop((0, FRAME_SIZE - 1, FRAME_SIZE, FRAME_SIZE)).getdata())
        border.extend(alpha.crop((0, 0, 1, FRAME_SIZE)).getdata())
        border.extend(alpha.crop((FRAME_SIZE - 1, 0, FRAME_SIZE, FRAME_SIZE)).getdata())
        if any(border):
            errors.append(f"frame {index} touches its cell border and may bleed")
        frame_hashes.append(hashlib.sha256(frame.tobytes()).hexdigest())

    unique_frames = len(set(frame_hashes))
    if unique_frames < 8:
        errors.append(f"expected at least 8 distinct frames, got {unique_frames}")
    opaque_colors = {
        (red, green, blue)
        for red, green, blue, alpha in image.getdata()
        if alpha == 255
    }
    if len(opaque_colors) > 64:
        errors.append(f"opaque palette exceeds 64 colors: {len(opaque_colors)}")
    if errors:
        raise ValueError("; ".join(errors))

    resolved = path.resolve()
    try:
        asset = resolved.relative_to(ROOT).as_posix()
    except ValueError:
        asset = resolved.name

    return {
        "asset": asset,
        "size": list(image.size),
        "frames": FRAME_COUNT,
        "uniqueFrames": unique_frames,
        "opaqueColors": len(opaque_colors),
        "coverage": coverage,
        "status": "verified",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sheet", type=Path, default=DEFAULT_SHEET)
    args = parser.parse_args()
    print(json.dumps(validate_sheet(args.sheet), indent=2))


if __name__ == "__main__":
    main()
