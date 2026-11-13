# AGENTS.md — OpenFamiliar

These instructions bind all automated contributors in this repository.

## Purpose

OpenFamiliar is a Windows 11 x64 pixel-art desktop companion. Stable v0.1
runs Perrito Tech in a transparent Tauri 2 window and submits one independent
question at a time through an existing authenticated Codex CLI installation.

The monorepo also holds experimental crates, TypeScript SDKs, adapter stubs,
Creator Studio, and a VS Code extension. Those are scaffolds or future work.
They are not product capabilities of the stable build unless an accepted ADR
says otherwise. See [ADR-009](docs/adr/ADR-009-windows-one-shot-mvp.md).

A proposed expansion of one-shot providers is recorded in
[ADR-010](docs/adr/ADR-010-additional-one-shot-providers.md). Until that ADR
is accepted and the matching code ships, do not treat Anthropic, OpenCode, or
Antigravity as stable desktop features.

## Quick map

| Path                               | Role                                                                  |
| ---------------------------------- | --------------------------------------------------------------------- |
| `apps/desktop/`                    | Stable Windows Tauri 2 + React application                            |
| `apps/desktop/src-tauri/`          | Rust backend of the stable app (`openfamiliar-desktop`)               |
| `packages/mascot-runtime/`         | Pixel-art animation runtime used by the desktop UI                    |
| `packages/provider-sdk/`           | One-shot and experimental provider TypeScript contracts               |
| `mascots/perrito-tech/`            | Canonical bundled pack                                                |
| `tools/familiar-cli/`              | Pack CLI (`familiar pack init/validate/build/inspect`)                |
| `crates/`                          | Experimental shared Rust services (not started by the stable desktop) |
| `adapters/`                        | Experimental TypeScript adapter stubs                                 |
| `docs/adr/`                        | Architecture Decision Records                                         |
| `.agents/skills/`                  | Repo-local reusable skills                                            |
| `docs/development/verification.md` | Local and CI verification, including the license inventory            |

A crate and package map, with internal dependencies and a mermaid diagram, is
in [docs/architecture/modules.md](docs/architecture/modules.md).

## Hard rules

- Do not change `apps/desktop/src-tauri/capabilities/default.json`, the CSP in
  `apps/desktop/src-tauri/tauri.conf.json`, or files under `.github/workflows/`
  unless the change is explicitly justified in the pull request. A template or
  Markdown file under `.github/` is not a workflow.
- Do not add remote fonts or remote visual assets to the stable app.
- New dependencies must use an exact version and a license compatible with
  [docs/legal/allowed-licenses.md](docs/legal/allowed-licenses.md). Update
  `docs/legal/dependency-licenses.json` from Windows when lockfiles change.
- Never commit, log, or fixture secrets, API keys, tokens, or credentials.
  Never hardcode a user, machine, checkout, executable, or log path.
- API keys belong only in the OS credential store (Windows Credential Manager
  first). Never store them in config files, preferences JSON, `.env` used at
  runtime, or source. The stable desktop today does not store provider keys;
  any new key I/O must follow [ADR-005](docs/adr/ADR-005-secure-credential-storage.md).
- Never read Codex, OpenCode, Antigravity, or browser auth files, sessions,
  cookies, or tokens.
- Never log prompts, answers, full stdout/stderr, credentials, or tokens.
- Stable IPC must expose purpose-specific commands only. Never add generic
  shell, command, path, argument, provider, workspace, or agent execution.
- Codex questions must be ephemeral, read-only, timeout-bounded, output-capped,
  cancellable, and process-tree terminated on Windows.
- Automated tests must use fake CLIs or mocks and must never call a real
  provider or consume quota.
- Preserve pixel-perfect integer scaling and nearest-neighbor rendering.
- Do not add circular avatar crops, permanent glow rings, technical state
  badges, fractional scaling, or third-party characters, sprites, logos, or
  branded proportions.
- Preserve Apache-2.0 for code, CC BY 4.0 for Perrito Tech, and CC0 where
  stated.

## Product boundary

- Target Windows 11 x64 first.
- Keep other providers, workspaces, agents, MCP, Creator Studio, and the VS
  Code extension experimental. Do not initialize or expose them in the stable
  build unless an accepted ADR authorizes that change.
- Prefer small reuse-first changes. Do not rewrite the monorepo or duplicate an
  existing abstraction.

## Working agreement

- Check `git status --short --branch` before edits and preserve unrelated work.
- Keep all project automation, skills, docs, fixtures, and generated
  deliverables inside this repository. Do not place project files in sibling
  workspaces.
- Do not commit, push, publish, sign, or open a PR unless the operator
  explicitly requests that action. Never push automatically.
- Do not delete future-phase scaffolds unless removal is necessary and
  approved.
- Architectural product changes need an ADR under `docs/adr/`. Start from the
  [ADR index](docs/adr/README.md).

## Validation

Run the narrowest relevant checks, then the full release set before packaging.
The exact local and CI matrix, including the license-inventory sort difference
between Windows and Linux, is in
[docs/development/verification.md](docs/development/verification.md).

```powershell
pnpm install --frozen-lockfile
pnpm format:check
pnpm typecheck
pnpm test
pnpm build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p familiar-cli -- pack validate mascots/perrito-tech
pnpm --filter @openfamiliar/desktop tauri build
```

If a tool component is unavailable, report that exact limitation; do not claim
the check passed.
