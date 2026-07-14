# Perrito Tech

Official Windows MVP familiar pack for OpenFamiliar.

- Asset: `assets/perrito-tech-spritesheet.png`
- Grid: 4×4 frames, 64×64 px per frame
- Rendering: integer nearest-neighbor scale only (1×, 2× or 3×)
- Default desktop rendering: 2× (128×128 px)
- Stable idle: one static frame; animation is triggered only by explicit events
- Appearance: happy, natural, and collar-free in every frame
- Variants: teal default, midnight, and burgundy state-prop accent palettes
- Animation ceiling: 12 FPS
- License: CC-BY-4.0 (art) — see `LICENSE` and `NOTICE`
- Engine: >=0.1.0

The legacy `.webp` marker files remain only for pack-compatibility fixtures and
must not be used by the desktop runtime.

Rebuild or verify deterministic palette variants:

```powershell
pnpm assets:variants
pnpm assets:check
```
