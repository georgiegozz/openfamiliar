# Third-Party Notices

This file records redistributed dependencies and conceptual references. It is
not a replacement for the licenses shipped with each dependency.

## Policy

- Core code: Apache-2.0 or a reviewed compatible license.
- GPL/AGPL-only dependencies are blocked for the core by default.
- Unlicensed, all-rights-reserved, personal-use-only, or non-commercial assets
  are not copied or redistributed.

## Conceptual references — no code or art copied

| Project       | Observed license boundary                | Use in OpenFamiliar                              |
| ------------- | ---------------------------------------- | ------------------------------------------------ |
| BongoCat      | MIT repository                           | Tauri desktop-pet and custom-model UX reference  |
| Petdex        | MIT repository                           | Manifest and spritesheet pack concept reference  |
| Clawd on Desk | AGPL-3.0 code; mixed/reserved art notice | Reference only; no source or assets incorporated |

OpenFamiliar uses its own manifest, runtime, UI, and original Perrito Tech art.

## Runtime dependency inventory

`docs/legal/dependency-licenses.json` is generated deterministically from the
current lockfiles with:

```powershell
pnpm licenses:audit
```

The current production JavaScript inventory contains MIT-licensed packages.
The Rust inventory contains SPDX expressions reported by Cargo metadata; no
`NOASSERTION`, GPL-only, or AGPL-only entry was found in this lockfile audit.
Some expressions offer multiple alternatives and must be reviewed under the
license selected by the distributed package. `Cargo.lock` and
`pnpm-lock.yaml` remain the dependency-version sources of truth.

## Trademarks

Provider names may appear as plain text for interoperability. Their logos and
brand marks are not part of the OpenFamiliar identity pack.
