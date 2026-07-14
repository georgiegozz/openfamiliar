---
name: openfamiliar-codex-adapter
description: Implement, harden, or test OpenFamiliar's Windows one-shot Codex CLI adapter. Use for process spawning, CLI capability detection, ephemeral read-only requests, output limits, cancellation, timeout, error classification, or privacy-safe logging in apps/desktop/src-tauri.
---

# OpenFamiliar Codex Adapter

## Security Boundary

The stable MVP may execute only a resolved `codex.exe`. Never accept an
arbitrary command, shell fragment, additional CLI arguments, working directory,
environment secret, or provider API key from the frontend.

Each request must use a fresh `codex exec` process with ephemeral state,
read-only sandboxing, approval disabled, no repository dependency, and the
prompt delivered through guarded stdin. Use the application's neutral local
working directory.

## Inputs

- Installed `codex.exe` capability output, or the repository fake CLI fixture.
- Requested adapter behavior and exact safe-error acceptance criteria.

## Workflow

1. Probe version, root help, exec help, and login status before enabling Ask.
2. Pass arguments as separate process arguments. Never concatenate a command
   line or invoke through `cmd.exe`/PowerShell in production.
3. Clear sensitive environment variables before spawning.
4. Capture stdout and stderr concurrently with hard byte ceilings.
5. Return stdout only after UTF-8 validation and ANSI removal. Never include raw
   stderr/stdout in user errors or logs.
6. On cancel or timeout, terminate the entire Windows process tree and remove
   the request from the active-process registry.
7. Classify missing CLI, unauthenticated, incompatible, rate limit, timeout,
   cancellation, oversized output, invalid output, and non-zero exit.

## Acceptance Criteria

- Only `codex.exe` can execute, with fixed safe arguments and no repo context.
- Timeout/cancel removes the full process tree; outputs are bounded and UTF-8.
- No prompt, answer, raw stdout/stderr, secret, or runtime path reaches logs/errors.

## Tests

Use only `apps/desktop/src-tauri/tests/fixtures/fake-codex.ps1` for process
integration tests. Cover Unicode, ANSI, non-zero exit, stdout/stderr behavior,
oversized output, timeout, and a spawned child process.

```powershell
cargo test -p openfamiliar-desktop services::codex_process::tests
cargo check -p openfamiliar-desktop --all-targets
```

## Limits and Anti-patterns

Never call real Codex in tests, concatenate shell text, accept freeform args,
inherit API-key variables, read auth/session files, or kill only the parent.

Relevant files: `codex_process.rs`, `logging.rs`, `commands/mod.rs`, the fake CLI
fixture, and `packages/provider-sdk`.
