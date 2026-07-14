"""Build deterministic prop-accent variants from a canonical RGBA sheet."""

from __future__ import annotations

import argparse
import io
import json
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CONFIG = ROOT / "mascots" / "perrito-tech" / "palette-variants.json"


def parse_hex(value: str) -> tuple[int, int, int]:
    normalized = value.removeprefix("#")
    if len(normalized) != 6:
        raise ValueError(f"Expected #RRGGBB color, got {value!r}")
    return tuple(int(normalized[index : index + 2], 16) for index in (0, 2, 4))


def repo_path(value: str) -> Path:
    candidate = (ROOT / value).resolve()
    try:
        candidate.relative_to(ROOT)
    except ValueError as error:
        raise ValueError(f"Path must stay inside the repository: {value!r}") from error
    return candidate


def build(config_path: Path, check: bool = False) -> list[dict[str, object]]:
    config = json.loads(config_path.read_text(encoding="utf-8"))
    source = repo_path(config["source"])
    source_palette = {
        name: parse_hex(color) for name, color in config["sourcePalette"].items()
    }
    image = Image.open(source).convert("RGBA")
    if image.size != (256, 256):
        raise ValueError(f"Expected a 256x256 sheet, got {image.size!r}")
    if image.getchannel("A").getextrema() != (0, 255):
        raise ValueError("Canonical sheet must contain transparent and opaque pixels")
    if any(alpha not in (0, 255) for *_, alpha in image.getdata()):
        raise ValueError("Canonical sheet must use binary alpha")

    source_pixels = list(image.getdata())
    results: list[dict[str, object]] = []
    for name, definition in config["variants"].items():
        palette = {
            source_palette[key]: parse_hex(definition["palette"][key])
            for key in source_palette
        }
        output_pixels = []
        changed = 0
        for red, green, blue, alpha in source_pixels:
            replacement = palette.get((red, green, blue))
            if replacement is not None and alpha:
                red, green, blue = replacement
                changed += 1
            output_pixels.append((red, green, blue, alpha))
        if changed == 0:
            raise ValueError(f"Variant {name!r} did not match any accent pixels")

        output = repo_path(definition["asset"])
        variant = Image.new("RGBA", image.size)
        variant.putdata(output_pixels)
        encoded = io.BytesIO()
        variant.save(encoded, format="PNG", optimize=True)
        expected = encoded.getvalue()
        if check:
            if not output.is_file() or output.read_bytes() != expected:
                raise ValueError(f"Variant {name!r} is missing or stale: {output}")
        else:
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_bytes(expected)
        results.append(
            {
                "variant": name,
                "asset": output.relative_to(ROOT).as_posix(),
                "changedPixels": changed,
                "status": "verified" if check else "written",
            }
        )
    return results


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    config_path = args.config.resolve()
    config_path.relative_to(ROOT)
    print(json.dumps(build(config_path, check=args.check), indent=2))


if __name__ == "__main__":
    main()
