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

```powershell
corepack prepare pnpm@9.15.0 --activate
pnpm install --frozen-lockfile
py -3.12 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r .\scripts\assets\requirements.txt
rustup component add rustfmt clippy
pnpm ci:js
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm validate:packs
```

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
