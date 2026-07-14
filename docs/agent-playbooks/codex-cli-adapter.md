# Codex CLI Adapter Playbook

## Use When

Changing Codex discovery, compatibility probes, one-shot execution, stdin,
timeouts, cancellation, process trees, output parsing, errors, or safe logging.

## Inputs

- Installed `codex.exe` or the fake fixture.
- Output of `codex --version`, `codex --help`, `codex exec --help`, and
  `codex login status` for compatibility decisions.

## Steps

1. Resolve only `codex.exe` from an optional validated absolute path or PATH.
2. Probe required flags and authentication without reading auth storage.
3. Spawn directly with separate arguments, neutral cwd, and sensitive env removed.
4. Send a bounded guarded prompt through stdin.
5. Capture stdout/stderr concurrently with byte limits and strip ANSI.
6. On timeout/cancel, terminate the full Windows process tree.
7. Return a safe category and generic message; log category-only events.

## Acceptance Criteria

- Fresh ephemeral read-only process per request; no repository context or memory.
- Missing, unauthenticated, incompatible, rate-limited, timeout, cancel, invalid,
  oversized, and non-zero outcomes are differentiated.
- Raw prompt, answer, stdout, and stderr never appear in logs or errors.

## Validation

```powershell
cargo test -p openfamiliar-desktop services::codex_process::tests
cargo check -p openfamiliar-desktop --all-targets
```

## Limits and Anti-patterns

Never call the real service in tests, concatenate a shell command, expose freeform
args, inherit API-key variables, read auth/session files, or kill only the parent.

Relevant files: `codex_process.rs`, `logging.rs`, `commands/mod.rs`,
`tests/fixtures/fake-codex.ps1`, `packages/provider-sdk`.
