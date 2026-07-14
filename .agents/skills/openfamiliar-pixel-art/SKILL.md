---
name: openfamiliar-pixel-art
description: Create, integrate, or review OpenFamiliar pixel-art mascot assets and animation manifests. Use for spritesheets, animation states, ambient behavior, integer scaling, transparency, asset licensing, or changes to packages/mascot-runtime and mascots/perrito-tech.
---

# OpenFamiliar Pixel Art

## Canonical Asset

Use `mascots/perrito-tech/assets/perrito-tech-spritesheet.png` as the only
desktop Perrito Tech sprite source. It is a transparent 4×4 sheet with 96×96
pixel cells. Do not copy it into `apps/desktop/public`.

## Workflow

1. Inspect the canonical PNG and `mascots/perrito-tech/familiar.json`.
2. Map every runtime animation to valid zero-based frame indices.
3. Keep animation rates between 1 and 12 FPS. Prefer 2–8 FPS.
4. Render only at integer 1×, 2×, or 3× scale with nearest-neighbor sampling.
5. Keep idle ambient actions low-frequency and interruptible. Operational
   states must take priority over ambient actions.
6. Pause animation while the document is hidden and honor reduced motion.
7. Update `README.md`, `LICENSE`, `NOTICE`, `assetSources`, and `aiGenerated`
   whenever the source or production workflow changes.

## Required States

Support idle, listening, thinking, answering, success, error, sleeping, wake,
and dragging. Ambient actions may include breathe, blink, directional looks,
ear twitch, and a rare special idle.

## Validation

```powershell
pnpm --filter @openfamiliar/mascot-runtime test
pnpm validate:packs
pnpm --filter @openfamiliar/desktop build
```

Visually verify transparent edges, frame alignment, and nearest-neighbor scale
before release.
