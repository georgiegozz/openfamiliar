from __future__ import annotations

import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from PIL import Image, ImageDraw


ASSET_SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ASSET_SCRIPTS))

from validate_mascot_sheet import validate_sheet  # noqa: E402


def valid_sheet() -> Image.Image:
    image = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    for index in range(16):
        left = (index % 4) * 64 + 12
        top = (index // 4) * 64 + 12
        color = (40 + index * 8, 80, 100, 255)
        draw.rectangle((left, top, left + 30, top + 30), fill=color)
    return image


class ValidateMascotSheetTests(unittest.TestCase):
    def validate_image(self, image: Image.Image) -> dict[str, object]:
        with TemporaryDirectory() as directory:
            path = Path(directory) / "sheet.png"
            image.save(path)
            return validate_sheet(path)

    def test_accepts_well_formed_grid(self) -> None:
        self.assertEqual(self.validate_image(valid_sheet())["frames"], 16)

    def test_rejects_partial_alpha(self) -> None:
        image = valid_sheet()
        image.putpixel((20, 20), (1, 2, 3, 128))
        with self.assertRaisesRegex(ValueError, "partial"):
            self.validate_image(image)

    def test_rejects_empty_frame(self) -> None:
        image = valid_sheet()
        ImageDraw.Draw(image).rectangle((0, 0, 63, 63), fill=(0, 0, 0, 0))
        with self.assertRaisesRegex(ValueError, "frame 0"):
            self.validate_image(image)

    def test_rejects_cell_border_bleed(self) -> None:
        image = valid_sheet()
        image.putpixel((0, 20), (10, 20, 30, 255))
        with self.assertRaisesRegex(ValueError, "border"):
            self.validate_image(image)


if __name__ == "__main__":
    unittest.main()
