# OpenFamiliar

**OpenFamiliar v0.1 is a Windows pixel-art desktop companion with one-shot Codex CLI questions.**

Perrito Tech lives in a transparent always-on-top window. Click the mascot, ask
one independent question, receive the answer from an existing authenticated
Codex CLI installation, and discard the interaction. There is no conversation
history, workspace context, memory, autonomous execution, or API-key storage.

> OpenFamiliar is an independent open-source project and is not affiliated with,
> sponsored by, or endorsed by OpenAI or any other tool vendor.

## Stable v0.1

| Capability                                          | Status                                           |
| --------------------------------------------------- | ------------------------------------------------ |
| Windows 11 x64 Tauri desktop                        | Implemented; local automated validation          |
| Perrito Tech 96×96 pixel-art sprites                | Implemented; CC BY 4.0                           |
| Transparent mascot and separate Settings window     | Implemented                                      |
| One-shot Quick Ask through Codex CLI                | Implemented with fake-CLI tests                  |
| Timeout, cancellation, process-tree cleanup         | Implemented with fake-CLI tests                  |
| System tray and click-through recovery              | Implemented                                      |
| Non-sensitive preferences and monitor-safe position | Implemented with unit tests                      |
| MSI and NSIS installers                             | Build configured; see release validation results |
| Clean-user Windows smoke test                       | Operator step; not implied by compilation        |

The stable runtime is local-first and idle-offline. It does not read Codex auth
storage; it invokes the official CLI already installed and authenticated by the
operator. Questions and answers remain in memory only. Operational logs contain
event categories, not content.

## Experimental / Future

The monorepo retains scaffolds for other providers, workspaces, permissioned
agents, MCP, Creator Studio, VS Code integration, advanced packs, and shared core
services. They are not initialized or visible in the stable Windows MVP and must
not be described as finished product capabilities. See [ROADMAP.md](./ROADMAP.md).

## Repository Layout

```text
apps/desktop/              stable Windows Tauri + React application
packages/mascot-runtime/   pixel-art animation runtime
packages/mascot-sdk/       familiar pack types
packages/provider-sdk/     one-shot provider contract + experimental contract
mascots/perrito-tech/      canonical spritesheet, manifest, and art license
crates/                    experimental/future shared Rust services
adapters/                  experimental/future provider and CLI scaffolds
docs/                      ADRs, playbooks, security, testing, and guides
.agents/skills/            repo-local Codex skills
```

## Prerequisites

- Windows 11 x64
- Node.js 20+
- pnpm 9.15.0 (declared by `packageManager`)
- Rust stable with `rustfmt` and `clippy`
- Visual Studio Build Tools 2022: Desktop development with C++
- WebView2 Runtime
- Codex CLI installed and authenticated for real Quick Ask use

## Develop

```powershell
# Run from the repository root.
corepack enable
corepack prepare pnpm@9.15.0 --activate
pnpm install --frozen-lockfile
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
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm validate:packs
```

## Build Windows Installers

```powershell
pnpm --filter @openfamiliar/desktop tauri build
```

Unsigned MSI and NSIS artifacts are produced below
`target/release/bundle/`. Code signing and publication are deferred; a successful
build is not a substitute for the clean-user checklist in
[WINDOWS_MVP_SMOKE_TEST.md](./docs/testing/WINDOWS_MVP_SMOKE_TEST.md).

## Security and Privacy

- No generic shell command or user-supplied CLI arguments.
- Fresh ephemeral, read-only Codex process per question.
- Prompt input and output size limits, timeout, cancellation, and process-tree kill.
- Restrictive Tauri CSP and minimum capability allowlist.
- No auth/session scraping, `.env` runtime loading, loopback listener, MCP, or remote UI.
- No prompt, answer, credential, token, or raw process output in logs.

See [ADR-009](./docs/adr/ADR-009-windows-one-shot-mvp.md) and the
[security boundary playbook](./docs/agent-playbooks/security-boundaries.md).

## License

- Code: [Apache License 2.0](./LICENSE)
- Perrito Tech artwork: CC BY 4.0
- Blank templates: CC0-1.0

See [NOTICE](./NOTICE), [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md), and
the Perrito Tech [asset notice](./mascots/perrito-tech/NOTICE).
