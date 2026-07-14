"""Normalize an AI-assisted contact sheet into the canonical pixel-art format."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path

from PIL import Image


CANONICAL_SIZE = (256, 256)


def normalize(input_path: Path, output_path: Path, colors: int = 64) -> dict[str, object]:
    source = Image.open(input_path).convert("RGBA")
    if source.width != source.height or source.width < CANONICAL_SIZE[0]:
        raise ValueError(f"Expected a square source at least 256px wide, got {source.size!r}")

    resized = source.resize(CANONICAL_SIZE, Image.Resampling.NEAREST)
    quantized = resized.quantize(
        colors=colors,
        method=Image.Quantize.FASTOCTREE,
        dither=Image.Dither.NONE,
    ).convert("RGBA")
    pixels = []
    for red, green, blue, alpha in quantized.getdata():
        if alpha < 128:
            pixels.append((0, 0, 0, 0))
        else:
            pixels.append((red, green, blue, 255))
    quantized.putdata(pixels)

    encoded = io.BytesIO()
    quantized.save(encoded, format="PNG", optimize=True)
    payload = encoded.getvalue()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(payload)
    return {
        "input": input_path.as_posix(),
        "output": output_path.as_posix(),
        "size": list(quantized.size),
        "opaquePixels": sum(1 for *_, alpha in pixels if alpha == 255),
        "sha256": hashlib.sha256(payload).hexdigest(),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--colors", type=int, default=64)
    args = parser.parse_args()
    if not 16 <= args.colors <= 256:
        raise ValueError("--colors must be between 16 and 256")
    print(json.dumps(normalize(args.input, args.output, args.colors), indent=2))


if __name__ == "__main__":
    main()
