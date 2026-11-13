# Module and crate map

Verified against the manifests and imports on `main` (`8afa609` and this docs
branch). Status labels follow [ROADMAP.md](../../ROADMAP.md): **stable** means
the v0.1 desktop path; **experimental** or **scaffold** means the code exists
in the monorepo but is not initialized or shown by the stable app.

Internal dependencies listed here are workspace/path dependencies or
`@openfamiliar/*` package imports actually found in source. Transitive crates
from crates.io are omitted.

## Apps

| Path                      | Package                        | Responsibility                                                                                               | Internal dependencies                                                | Status   |
| ------------------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------- | -------- |
| `apps/desktop/`           | `@openfamiliar/desktop`        | Stable Windows UI: mascot window, Quick Ask, Settings, tray                                                  | `@openfamiliar/mascot-runtime`, `@openfamiliar/provider-sdk` (types) | stable   |
| `apps/desktop/src-tauri/` | `openfamiliar-desktop`         | Tauri 2 host: preferences, window/tray, Codex one-shot process, safe logging                                 | none of the `familiar-*` crates                                      | stable   |
| `apps/creator-studio/`    | `@openfamiliar/creator-studio` | Vite + React scaffold that edits pack metadata JSON; it does not load images or export a `.familiar` archive | none                                                                 | scaffold |
| `apps/vscode-extension/`  | `openfamiliar-companion`       | Thin editor-bridge scaffold; its README says API keys stay in the desktop app                                | none                                                                 | scaffold |

The stable Rust host registers only purpose-specific commands (`detect_codex`,
`ask_codex`, `cancel_codex`, preferences, window, and quit). The frontend
wrapper is `apps/desktop/src/lib/backend.ts`. There is no generic shell,
workspace, provider-router, or agent command on this path.

`openfamiliar-desktop` does not depend on `familiar-core`,
`familiar-storage`, `familiar-permissions`, `familiar-context`,
`familiar-agent-bridge`, or `familiar-mcp`. That was confirmed in
`apps/desktop/src-tauri/Cargo.toml` and by a source search under `apps/desktop/`.

## TypeScript packages

Workspace members come from `pnpm-workspace.yaml`: `apps/*`, `packages/*`,
`adapters/*`, and `tools/*`.

| Path                       | Package                        | Responsibility                                                                                       | Internal dependencies | Status                                     |
| -------------------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------- | --------------------- | ------------------------------------------ |
| `packages/mascot-runtime/` | `@openfamiliar/mascot-runtime` | Frame ticking and Perrito Tech animation maps; FPS clamped to 1–12                                   | none                  | stable                                     |
| `packages/provider-sdk/`   | `@openfamiliar/provider-sdk`   | `OneShotProvider` used as types by the desktop, plus an experimental `ModelProvider` stream contract | none                  | stable types; `ModelProvider` experimental |
| `packages/mascot-sdk/`     | `@openfamiliar/mascot-sdk`     | TypeScript types for pack manifests and sprite sheets                                                | none                  | present; no app import found               |
| `packages/agent-sdk/`      | `@openfamiliar/agent-sdk`      | TypeScript agent event types only                                                                    | none                  | experimental; no app import found          |
| `packages/schemas/`        | `@openfamiliar/schemas`        | `familiar-v1.json` and a Node test that loads it                                                     | none                  | tooling                                    |
| `packages/ui/`             | `@openfamiliar/ui`             | Single helper (`formatTokens`)                                                                       | peer `react`          | present; no app import found               |

Pack JSON files point at `https://openfamiliar.dev/schema/familiar-v1.json`.
`familiar-cli` validates packs in Rust and does not load this JSON Schema file
at runtime. Whether that URL is served on the public site was not verified
here.

## Rust crates and tools

Workspace members are listed in the root `Cargo.toml`.
`rust-version` is `1.80`. `openfamiliar-desktop` is a member but is not in
`default-members`.

| Path                            | Crate                              | Responsibility                                                                                                                                                             | Internal dependencies                                          | Status                                          |
| ------------------------------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- | ----------------------------------------------- |
| `crates/familiar-core/`         | `familiar-core`                    | Session/event orchestration and a `ModelProvider` router (Ollama, OpenAI-compatible, Gemini). No Anthropic adapter. Router may read `OPENFAMILIAR_*` environment variables | `familiar-storage`, `familiar-permissions`, `familiar-context` | experimental                                    |
| `crates/familiar-storage/`      | `familiar-storage`                 | SQLite, audit log, path helpers, and a `keyring`-backed `CredentialStore`                                                                                                  | none                                                           | experimental                                    |
| `crates/familiar-permissions/`  | `familiar-permissions`             | Permission broker types and approval scopes                                                                                                                                | `familiar-storage`                                             | experimental                                    |
| `crates/familiar-context/`      | `familiar-context`                 | Read-only workspace context and a token-budget helper                                                                                                                      | none                                                           | experimental                                    |
| `crates/familiar-agent-bridge/` | `familiar-agent-bridge`            | Subprocess `AgentAdapter` scaffolds: Codex (`codex exec <task>`), OpenCode (`opencode run <task>`), Antigravity (program `antigravity` plus the task), Gemini CLI          | `familiar-permissions`                                         | experimental                                    |
| `crates/familiar-mcp/`          | `familiar-mcp`                     | Local MCP tool/resource catalog. The crate file states it is stdio-oriented with no public bind                                                                            | `familiar-core`                                                | scaffold; no listener or startup in the desktop |
| `tools/familiar-cli/`           | `familiar-cli` (binary `familiar`) | `pack init`, `validate`, `preview`, `build`, and `inspect`. Pack size cap 50 MiB; executable files rejected                                                                | none                                                           | tooling                                         |

The Antigravity scaffold's program name and argv are whatever the current
source asserts: program `antigravity`, arguments `[task]`. Public CLI
documentation for a one-shot `agy -p` invocation is a product concern tracked
in [ADR-010](../adr/ADR-010-additional-one-shot-providers.md). This map does
not claim that the scaffold already matches that CLI.

## Adapter stubs

Each directory under `adapters/` is a one-line TypeScript stub (`adapterId`)
plus a README that points at Familiar Core or the agent bridge. They are not
imported by `apps/desktop/`.

| Path                          | Package                                   | `adapterId`         |
| ----------------------------- | ----------------------------------------- | ------------------- |
| `adapters/codex/`             | `@openfamiliar/adapter-codex`             | `codex`             |
| `adapters/ollama/`            | `@openfamiliar/adapter-ollama`            | `ollama`            |
| `adapters/openai-compatible/` | `@openfamiliar/adapter-openai-compatible` | `openai-compatible` |
| `adapters/gemini/`            | `@openfamiliar/adapter-gemini`            | `gemini`            |
| `adapters/gemini-cli/`        | `@openfamiliar/adapter-gemini-cli`        | `gemini-cli`        |
| `adapters/opencode/`          | `@openfamiliar/adapter-opencode`          | `opencode`          |
| `adapters/antigravity/`       | `@openfamiliar/adapter-antigravity`       | `antigravity`       |

## Mascots and scripts

| Path                                          | Responsibility                                                                     |
| --------------------------------------------- | ---------------------------------------------------------------------------------- |
| `mascots/perrito-tech/`                       | Canonical CC BY 4.0 pack bundled by the stable desktop, including palette variants |
| `mascots/blank-template/`                     | CC0 template validated by `pnpm validate:packs`                                    |
| `scripts/assets/`                             | Python/Pillow checks and deterministic palette-variant generation                  |
| `scripts/legal/Export-DependencyLicenses.ps1` | Regenerates `docs/legal/dependency-licenses.json`                                  |
| `scripts/test-all.ps1`                        | Local Windows check sweep (does not include `pnpm licenses:check` or `pnpm build`) |

## Security files the desktop actually enforces

These are part of the stable host, not a separate crate:

- Capabilities: `apps/desktop/src-tauri/capabilities/default.json` — windows
  `mascot` and `settings`; permissions `core:default` and
  `core:window:allow-start-dragging` only.
- CSP: `apps/desktop/src-tauri/tauri.conf.json` `app.security.csp` —
  `default-src 'self'`; images from `'self'`, `asset:`,
  `http://asset.localhost`, and `data:`; `connect-src ipc: http://ipc.localhost`;
  `frame-src 'none'`.

Changing either file, or any workflow under `.github/workflows/`, needs an
explicit justification in the pull request. See [AGENTS.md](../../AGENTS.md).
