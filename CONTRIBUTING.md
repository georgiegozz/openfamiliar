# Contributing to OpenFamiliar

Thanks for your interest in contributing.

## Ground rules

1. No secrets, API keys, tokens, or personal/client data in commits, fixtures, screenshots, or logs.
2. Synthetic examples only under `examples/`.
3. Every asset needs author, origin, and license metadata.
4. New dependencies require a license review against `docs/legal/allowed-licenses.md`.
5. Stable desktop IPC must not add a generic shell, arbitrary CLI arguments,
   provider selection, workspace access, or autonomous actions.
6. Architectural changes need an ADR under `docs/adr/`.
7. Use Conventional Commits and SemVer for releases.
8. Open a pull request even when working alone; keep `main` always buildable.

## Development setup

The workspace `rust-version` is 1.88.

Local and CI commands, including how to regenerate the license inventory on
Windows, are in [docs/development/verification.md](./docs/development/verification.md).
The pull request checklist lives in `.github/PULL_REQUEST_TEMPLATE.md`.
Agent rules and the module map start at [AGENTS.md](./AGENTS.md).
Windows toolchain notes also live in [docs/guides/dev-windows.md](./docs/guides/dev-windows.md).

## Repository Layout

```text
apps/desktop/              stable Windows Tauri + React application
packages/mascot-runtime/   pixel-art animation runtime
packages/mascot-sdk/       familiar pack types
packages/provider-sdk/     one-shot provider contract + experimental contract
mascots/perrito-tech/      canonical spritesheet, manifest, and art license
crates/                    experimental/future shared Rust services
adapters/                  experimental/future provider and CLI scaffolds
docs/                      ADRs, architecture map, verification, playbooks, security, testing, and guides
docs/development/          local and CI verification, including the license inventory
.agents/skills/            repo-local reusable skills
```

## Prerequisites

- Windows 11 x64
- Node.js 20+
- pnpm 9.15.0 (declared by `packageManager`)
- Rust 1.88+ with `rustfmt` and `clippy`
- Python 3.12 for deterministic mascot asset checks
- Visual Studio Build Tools 2022: Desktop development with C++
- WebView2 Runtime
- Codex CLI installed and authenticated for real Quick Ask use

## Develop

```powershell
# Run from the repository root.
corepack enable
corepack prepare pnpm@9.15.0 --activate
pnpm install --frozen-lockfile
py -3.12 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r .\scripts\assets\requirements.txt
rustup component add rustfmt clippy
pnpm --filter @openfamiliar/desktop tauri dev
```

Browser-only UI preview uses a non-network mock and cannot invoke Codex:

```powershell
pnpm --filter @openfamiliar/desktop dev
```

## Validate

Automated tests never call the real Codex service or consume quota.

```powershell
pnpm format:check
pnpm typecheck
pnpm test
pnpm build
pnpm assets:check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm validate:packs
pnpm licenses:audit
```

`pnpm ci:js` is the JavaScript CI bundle (format, typecheck, tests, asset checks,
pack validation, and `licenses:check`). After dependency changes, also review
the `pnpm licenses:audit` diff.

See [verification.md](./docs/development/verification.md) for the full Windows and Linux notes and for what each CI job runs.

## Build Windows Installers

```powershell
pnpm --filter @openfamiliar/desktop tauri build
```

Unsigned MSI and NSIS artifacts are produced below `target/release/bundle/`.
The source is suitable for a pre-1.0 public beta after the checks above pass. A
public binary must be signed or explicitly identified as an unsigned pre-release,
and it must pass the clean-user checklist in
[WINDOWS_MVP_SMOKE_TEST.md](./docs/testing/WINDOWS_MVP_SMOKE_TEST.md).

## Commit messages

```
feat(core): add permission broker session scope
fix(desktop): restore window position per monitor
docs(adr): accept ADR-004 provider abstraction
chore(ci): add secret scan step
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.

## Pull requests

- Target `main`.
- Link related issues.
- Include tests for changed runtime, pack, IPC, or process contracts.
- Update docs when behavior or schemas change.
- Run `pnpm assets:check` after changing Perrito Tech or palette variants.
- Run `pnpm licenses:audit` and review the diff after dependency changes.
- Do not copy third-party mascot art. Record provenance and asset licensing.
- Sign off with DCO (`Signed-off-by:`) — see `DCO`.

## Code of conduct

Participants must follow [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).

## License of contributions

By contributing, you agree your contributions are licensed under Apache-2.0
for code, unless an asset contribution is explicitly dual-licensed (e.g. CC-BY-4.0 art).
