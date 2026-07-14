---
name: openfamiliar-pixel-art
description: Create, integrate, recolor, or review OpenFamiliar pixel-art mascot assets and animation manifests. Use for spritesheets, event-driven animation states, palette variants, integer scaling, transparency, asset licensing, or changes to packages/mascot-runtime and mascots/perrito-tech.
---

# OpenFamiliar Pixel Art

## Canonical Asset

Use `mascots/perrito-tech/assets/perrito-tech-spritesheet.png` as the only
canonical Perrito Tech sprite source. It is a transparent 4×4 sheet with 64×64
pixel cells. Do not copy it into app public folders.

## Workflow

1. Inspect the canonical PNG and `mascots/perrito-tech/familiar.json`.
2. Map every runtime animation to valid zero-based frame indices.
3. Keep animation rates between 1 and 12 FPS. Prefer 2–8 FPS.
4. Render only at integer 1×, 2×, or 3× scale with nearest-neighbor sampling.
5. Keep stable idle to one symmetric frame. Never schedule ambient movement,
   independent pupil motion, random sleep, or random looks.
6. Pause animation while the document is hidden and honor reduced motion.
7. Define state-prop accent recolors in `palette-variants.json`, generate them
   with the repo script, and never hand-edit their output. Do not add or recolor
   neckwear; preserve the natural collar-free character.
8. Update `README.md`, `LICENSE`, `NOTICE`, `assetSources`, and `aiGenerated`
   whenever the source or production workflow changes.

## Required States

Support idle, listening, thinking, working, answering, approval, success,
error, sleeping, wake, and dragging. Multi-frame motion is event-driven only.

## Validation

```powershell
pnpm assets:check
pnpm --filter @openfamiliar/mascot-runtime test
pnpm validate:packs
pnpm --filter @openfamiliar/desktop build
```

Visually verify transparent edges, frame alignment, and nearest-neighbor scale
before release.
