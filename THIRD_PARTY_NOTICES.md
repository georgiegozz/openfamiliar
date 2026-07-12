# Third-Party Notices

This file lists third-party components that may be redistributed with OpenFamiliar
builds. Entries are added as dependencies are pinned and audited.

## Policy

- Core code: Apache-2.0 only (or compatible, after review).
- GPL / AGPL: blocked for the core by default.
- Unlicensed or All Rights Reserved: do not copy.
- Assets with “personal use only” or “non-commercial” terms: do not distribute.

## Conceptual references (no code copied)

| Project | License noted | Use in OpenFamiliar |
|---------|---------------|---------------------|
| OpenPets | MIT | Architecture study only |
| CoPet | MIT | Architecture study only (Tauri/React pet shell) |
| DeskPet | MIT (some assets excluded) | Security/provider ideas only |
| Clawd on Desk | AGPL-3.0 + reserved assets | Conceptual reference only — no code/assets |
| OpenCode Companion | License unclear | Conceptual reference only — no code/assets |
| Open-LLM-VTuber | MIT core + separate Live2D model licenses | Demonstrates code/art license split |

## Runtime dependencies

Populate this section when `cargo tree`, `pnpm licenses`, and SBOM generation
are run for a release candidate. Until then, treat lockfiles as source of truth.

### Rust (Cargo)

- To be filled from `Cargo.lock` on first release candidate.

### JavaScript / TypeScript (pnpm)

- To be filled from `pnpm-lock.yaml` on first release candidate.

## Trademarks

Provider names (OpenAI, Gemini, xAI, Ollama, Codex CLI, Gemini CLI) may appear
as plain text for interoperability. Logos and brand marks are not part of the
OpenFamiliar identity pack.
