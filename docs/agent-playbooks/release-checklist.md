# Windows Release Checklist

## Use When

Preparing, validating, or packaging a local Windows x64 MVP release.

## Inputs

- Reviewed local branch, frozen lockfiles, Rust lockfile, Tauri bundle metadata.
- A Windows 11 clean user or disposable VM for the human smoke test.

## Steps

1. Confirm clean dependency install and run format, type, JS, Rust, and pack checks.
2. Search tracked files for secrets, personal paths, diagnostic logs, obsolete
   stable-provider UI, generic shell IPC, and prompt/response logging.
3. Build both MSI and NSIS without signing.
4. Record installer paths, sizes, and SHA-256 hashes.
5. Install, run the complete Windows MVP smoke test, uninstall, and reinstall.
6. Commit only reviewed source and docs; never commit `target`, `dist`, or installers.

## Acceptance Criteria

- Both installer types exist and uninstall cleanly.
- No automated test consumes real Codex quota.
- All automated gates pass or have an explicit release-blocking limitation.
- Smoke-test evidence covers click-through recovery, cancellation, monitor/DPI,
  missing Codex onboarding, and application quit.

## Validation

```powershell
pnpm install --frozen-lockfile
pnpm ci:js
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @openfamiliar/desktop tauri build
```

## Limits and Anti-patterns

Do not sign, publish, upload, push, or call a build "clean-install tested" based
only on compilation. See `docs/testing/WINDOWS_MVP_SMOKE_TEST.md`.
