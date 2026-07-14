---
name: openfamiliar-release
description: Validate and package OpenFamiliar's Windows desktop MVP into local MSI and NSIS artifacts. Use for release readiness, dependency pinning, clean installation checks, smoke tests, artifact inventory, or CI and bundle configuration.
---

# OpenFamiliar Release

## Preconditions

Release only from a reviewed local branch with no secrets, personal absolute
paths, diagnostic logs, ignored build products, or unrelated changes staged.
Do not push, publish, sign, or upload without explicit authorization.

## Inputs

- Reviewed branch, frozen lockfiles, Tauri metadata, and current validation output.
- A clean Windows user or disposable VM for human installation testing.

## Validation Order

1. Install with the repository-pinned pnpm version and frozen lockfile.
2. Check formatting of tracked source, TypeScript, JavaScript tests, pack
   manifests, Rust tests, and `cargo check --all-targets`.
3. Confirm the Tauri capability file grants only required core/window actions.
4. Search for secrets, personal paths, obsolete providers, generic shell IPC,
   and prompt/response logging.
5. Run `pnpm --filter @openfamiliar/desktop tauri build` for both MSI and NSIS.
6. Inventory installers with file size and SHA-256.
7. Execute `docs/testing/WINDOWS_MVP_SMOKE_TEST.md` on a clean Windows user or
   equivalent disposable VM before labeling the release ready.

## Acceptance Criteria

- MSI and NSIS artifacts exist with size and SHA-256 inventory.
- Automated gates pass and real-Codex calls appear only in the human smoke test.
- Clean install/uninstall evidence is distinct from compile/build evidence.

## Release Gate

Block release when either installer is missing, the CLI boundary lacks
cancellation/timeout coverage, click-through has no tray recovery path, pack
attribution is incomplete, or stable UI exposes an experimental provider or
agent capability.

Record test limitations precisely. A successful compiler run is not a clean
installation test.

## Limits and Anti-patterns

Do not commit build products, conceal failed gates, sign with an invented
certificate, publish, upload, push, or call compilation a clean installation.

Relevant files: Tauri config/icons, lockfiles, CI workflows, release playbook,
and `docs/testing/WINDOWS_MVP_SMOKE_TEST.md`.
