from __future__ import annotations

import unittest
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[3]
CANONICAL = (
    ROOT / "mascots" / "perrito-tech" / "assets" / "perrito-tech-spritesheet.png"
)
VARIANTS = (
    ROOT / "mascots" / "perrito-tech" / "assets" / "variants"
)


class MascotVariantTests(unittest.TestCase):
    def test_variants_change_only_dragging_state_props(self) -> None:
        canonical = Image.open(CANONICAL).convert("RGBA")
        canonical_pixels = list(canonical.getdata())

        for name in ("midnight", "burgundy"):
            with self.subTest(variant=name):
                variant = Image.open(
                    VARIANTS / f"perrito-tech-{name}.png"
                ).convert("RGBA")
                self.assertEqual(variant.size, canonical.size)
                variant_pixels = list(variant.getdata())
                changed = [
                    index
                    for index, (source, replacement) in enumerate(
                        zip(canonical_pixels, variant_pixels, strict=True)
                    )
                    if source != replacement
                ]
                self.assertGreater(len(changed), 0)
                changed_frames = {
                    (index % 256) // 64 + 4 * ((index // 256) // 64)
                    for index in changed
                }
                self.assertEqual(changed_frames, {8})
                self.assertTrue(
                    all(
                        canonical_pixels[index][3] == variant_pixels[index][3]
                        for index in changed
                    )
                )


if __name__ == "__main__":
    unittest.main()
