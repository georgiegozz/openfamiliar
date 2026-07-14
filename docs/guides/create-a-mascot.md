# Create or recolor a mascot

The supported pack model is declarative: a `familiar.json` manifest plus image
assets under the same pack directory. Packs cannot execute JavaScript or native
code.

## New pack

```powershell
cargo run -p familiar-cli -- pack init my-buddy
# Add original/licensed files under my-buddy/assets and map states in familiar.json.
cargo run -p familiar-cli -- pack validate my-buddy
cargo run -p familiar-cli -- pack build my-buddy
cargo run -p familiar-cli -- pack inspect my-buddy.familiar
```

Record author, source, license, and AI assistance in the manifest plus a pack
`NOTICE`. Use only repo-owned or redistributable art. The current desktop bundles
Perrito Tech; installing arbitrary packs through the UI remains future work.

## Perrito Tech palette variants

Perrito Tech keeps one canonical transparent spritesheet. Approved recolors are
exact palette replacements for small state props only, such as the dragging
toy, and are defined in `mascots/perrito-tech/palette-variants.json`. The dog is
kept natural and collar-free in every variant.

```powershell
pnpm assets:variants
pnpm assets:check
pnpm validate:packs
```

`assets:variants` rebuilds teal-derived variants. `assets:check` validates the
canonical 4×4 sheet, runs validator regression tests, and performs a read-only
byte comparison that fails on variant drift. Do not recolor the coat, white
markings, eyes, neck, or transparency, and do not hand-edit generated variants.
