# Pixel Art Pipeline

## Use When

Adding or changing Perrito Tech artwork, frames, animations, ambient actions,
manifest metadata, scaling, transparency, or attribution.

## Inputs

- Original or approved AI-assisted source.
- Target 4×4 grid, 96×96 px frames, transparent background, CC BY 4.0.

## Steps

1. Verify the character is original and contains no third-party logo or identity.
2. Produce one canonical RGBA sheet under `mascots/perrito-tech/assets`.
3. Remove the chroma background and resize only with nearest-neighbor sampling.
4. Inspect every cell for edge contamination and alignment.
5. Map frames in both the pack manifest and mascot runtime.
6. Use 2–8 FPS normally, never more than 12 FPS.
7. Update README, LICENSE, NOTICE, provenance, and AI-generation metadata.

## Acceptance Criteria

- Transparent edges are clean; no circular crop, permanent glow, or text badge.
- 1×, 2×, and 3× render crisply.
- Required states and ambient actions stay within the sheet.

## Validation

```powershell
.\.venv\Scripts\python.exe .\scripts\assets\build_perrito_icons.py
pnpm --filter @openfamiliar/mascot-runtime test
cargo run -p familiar-cli -- pack validate mascots/perrito-tech
pnpm --filter @openfamiliar/desktop build
```

## Limits and Anti-patterns

Do not duplicate the sheet in app public folders, interpolate pixels, invent
attribution, replace source licenses, or ship temporary marker files as art.

Relevant files: `mascots/perrito-tech`, `packages/mascot-runtime`,
`packages/schemas/familiar-v1.json`, `apps/desktop/src/features/mascot`.
