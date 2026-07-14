"""Build Tauri icons from Perrito Tech's canonical first sprite frame."""

from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "mascots" / "perrito-tech" / "assets" / "perrito-tech-spritesheet.png"
OUTPUT = ROOT / "apps" / "desktop" / "src-tauri" / "icons"


FRAME_SIZE = 64


def centered_icon(frame: Image.Image, size: int, integer_scale: int) -> Image.Image:
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    edge = FRAME_SIZE * integer_scale
    sprite = frame.resize((edge, edge), Image.Resampling.NEAREST)
    offset = ((size - edge) // 2, (size - edge) // 2)
    canvas.alpha_composite(sprite, offset)
    return canvas


def main() -> None:
    sheet = Image.open(SOURCE).convert("RGBA")
    if sheet.size != (256, 256):
        raise ValueError(f"Expected a 256x256 sheet, got {sheet.size!r}")
    frame = sheet.crop((0, 0, FRAME_SIZE, FRAME_SIZE))
    OUTPUT.mkdir(parents=True, exist_ok=True)

    icon_32 = frame.resize((32, 32), Image.Resampling.NEAREST)
    icon_128 = centered_icon(frame, 128, 1)
    icon_512 = centered_icon(frame, 512, 7)

    icon_32.save(OUTPUT / "32x32.png", optimize=True)
    icon_128.save(OUTPUT / "128x128.png", optimize=True)
    icon_512.save(OUTPUT / "icon.png", optimize=True)
    icon_512.save(
        OUTPUT / "icon.ico",
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )


if __name__ == "__main__":
    main()
