---
name: openfamiliar-add-mascot-pack
description: Create, validate, or document an OpenFamiliar mascot pack. Use when adding a pack under mascots/, changing familiar.json, spritesheets, palette variants, or familiar-cli pack commands.
---

# Add a mascot pack

Packs are declarative: a `familiar.json` manifest plus assets. They cannot
contain JavaScript or native executables. The stable desktop currently
bundles only Perrito Tech. Installing arbitrary packs from the UI is still
future work; this skill covers repo packs and CLI validation.

Canonical format (verified in `scripts/assets/validate_mascot_sheet.py`,
`packages/schemas/familiar-v1.json`, and the pixel-art playbook): a
transparent 256x256 PNG sheet, 4x4 cells of 64x64, binary alpha, nearest
neighbor, animation FPS at most 12. See
`../openfamiliar-pixel-art/SKILL.md` for Perrito Tech art rules.

## Workflow

1. Start from `mascots/blank-template` or `familiar pack init`. Copy only
   original or redistributable art. Do not imitate third-party characters.
2. Fill `familiar.json` (`id`, `name`, `version`, `engine`, `author`,
   `license`, `personality`, `states`, assets). Record provenance in the
   pack `NOTICE` / `LICENSE` and in `assetSources` / `aiGenerated`.
3. Put images under the pack `assets/` directory. Keep the sheet aligned to
   the schema. For Perrito Tech palette variants, change
   `palette-variants.json` and generate with the repo script; do not
   hand-edit generated PNGs.
4. Validate with `familiar-cli`. Then run asset checks if you touched
   Perrito Tech.

## PowerShell

```powershell
cargo run -p familiar-cli -- pack init my-buddy
# Add original assets and edit my-buddy/familiar.json, LICENSE, NOTICE.
cargo run -p familiar-cli -- pack validate my-buddy
cargo run -p familiar-cli -- pack build my-buddy
cargo run -p familiar-cli -- pack inspect my-buddy.familiar
pnpm validate:packs
# If you changed Perrito Tech or its variants:
pnpm assets:check
```

## Python

```python
from pathlib import Path
import json

pack = Path("mascots/my-buddy")
manifest = json.loads((pack / "familiar.json").read_text(encoding="utf-8"))
required = ["id", "name", "version", "engine", "author", "license", "personality"]
missing = [key for key in required if not manifest.get(key)]
print("missing fields:", missing or "none")

allowed = {".webp", ".png", ".jpg", ".jpeg", ".gif", ".md", ".json", ".txt", ".wav", ".ogg", ".mp3"}
blocked = {".exe", ".dll", ".bat", ".cmd", ".ps1", ".sh", ".js", ".ts"}
for path in pack.rglob("*"):
    if path.is_file() and path.suffix.lower() in blocked:
        raise SystemExit(f"executable or script not allowed: {path}")
    if path.is_file() and path.suffix.lower() not in allowed and path.name not in {"LICENSE", "NOTICE"}:
        print("unexpected file:", path)
```

The CLI also rejects executables and caps a pack at 50 MiB
(`tools/familiar-cli/src/main.rs`). Trust the CLI as the authority, not this
helper alone.

## Acceptance checks

```powershell
cargo test -p familiar-cli
pnpm validate:packs
pnpm --filter @openfamiliar/schemas test
pnpm --filter @openfamiliar/mascot-runtime test
```

If the pack is meant to appear in the stable desktop, that is a product
change and needs its own decision. Do not quietly replace Perrito Tech as
the default.
